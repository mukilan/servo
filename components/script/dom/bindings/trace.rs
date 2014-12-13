/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! Utilities for tracing JS-managed values.
//!
//! The lifetime of DOM objects is managed by the SpiderMonkey Garbage
//! Collector. A rooted DOM object implementing the interface `Foo` is traced
//! as follows:
//!
//! 1. The GC calls `_trace` defined in `FooBinding` during the marking
//!    phase. (This happens through `JSClass.trace` for non-proxy bindings, and
//!    through `ProxyTraps.trace` otherwise.)
//! 2. `_trace` calls `Foo::trace()` (an implementation of `JSTraceable`).
//!     This is typically derived via a #[jstraceable] annotation
//! 3. For all fields, `Foo::trace()`
//!    calls `trace()` on the field.
//!    For example, for fields of type `JS<T>`, `JS<T>::trace()` calls
//!    `trace_reflector()`.
//! 4. `trace_reflector()` calls `trace_object()` with the `JSObject` for the
//!    reflector.
//! 5. `trace_object()` calls `JS_CallTracer()` to notify the GC, which will
//!    add the object to the graph, and will trace that object as well.
//!
//! The no_jsmanaged_fields!() macro adds an empty implementation of JSTraceable to
//! a datatype.

use dom::bindings::js::JS;
use dom::bindings::refcounted::Trusted;
use dom::bindings::utils::{Reflectable, Reflector, WindowProxyHandler};
use dom::node::{Node, TrustedNodeAddress};
use script_task::ScriptChan;

use collections::hash::{Hash, Hasher};
use geom::rect::Rect;
use html5ever::tree_builder::QuirksMode;
use http::headers::request::HeaderCollection as RequestHeaderCollection;
use http::headers::response::HeaderCollection as ResponseHeaderCollection;
use http::method::Method;
use js::jsapi::{JSObject, JSTracer, JS_CallTracer, JSTRACE_OBJECT};
use js::jsval::JSVal;
use js::rust::Cx;
use layout_interface::{LayoutRPC, LayoutChan};
use libc;
use msg::constellation_msg::{PipelineId, SubpageId, WindowSizeData};
use net::image_cache_task::ImageCacheTask;
use script_traits::ScriptControlChan;
use script_traits::UntrustedNodeAddress;
use servo_msg::compositor_msg::ScriptListener;
use servo_msg::constellation_msg::ConstellationChan;
use servo_util::smallvec::{SmallVec1, SmallVec};
use servo_util::str::LengthOrPercentageOrAuto;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::comm::{Receiver, Sender};
use std::intrinsics::return_address;
use std::io::timer::Timer;
use std::rc::Rc;
use string_cache::{Atom, Namespace};
use style::PropertyDeclarationBlock;
use url::Url;


/// A trait to allow tracing (only) DOM objects.
pub trait JSTraceable {
    /// Trace `self`.
    fn trace(&self, trc: *mut JSTracer);
}

impl<T: Reflectable> JSTraceable for JS<T> {
    fn trace(&self, trc: *mut JSTracer) {
        trace_reflector(trc, "", self.reflector());
    }
}

no_jsmanaged_fields!(Reflector)

/// Trace a `JSVal`.
pub fn trace_jsval(tracer: *mut JSTracer, description: &str, val: JSVal) {
    if !val.is_markable() {
        return;
    }

    unsafe {
        let name = description.to_c_str();
        (*tracer).debugPrinter = None;
        (*tracer).debugPrintIndex = -1;
        (*tracer).debugPrintArg = name.as_ptr() as *const libc::c_void;
        debug!("tracing value {:s}", description);
        JS_CallTracer(tracer, val.to_gcthing(), val.trace_kind());
    }
}

/// Trace the `JSObject` held by `reflector`.
#[allow(unrooted_must_root)]
pub fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    trace_object(tracer, description, reflector.get_jsobject())
}

/// Trace a `JSObject`.
pub fn trace_object(tracer: *mut JSTracer, description: &str, obj: *mut JSObject) {
    unsafe {
        let name = description.to_c_str();
        (*tracer).debugPrinter = None;
        (*tracer).debugPrintIndex = -1;
        (*tracer).debugPrintArg = name.as_ptr() as *const libc::c_void;
        debug!("tracing {:s}", description);
        JS_CallTracer(tracer, obj as *mut libc::c_void, JSTRACE_OBJECT);
    }
}

impl<T: JSTraceable> JSTraceable for RefCell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        self.borrow().trace(trc)
    }
}

impl<T: JSTraceable> JSTraceable for Rc<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

impl<T: JSTraceable> JSTraceable for Box<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

impl<T: JSTraceable+Copy> JSTraceable for Cell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        self.get().trace(trc)
    }
}

impl JSTraceable for *mut JSObject {
    fn trace(&self, trc: *mut JSTracer) {
        trace_object(trc, "object", *self);
    }
}

impl JSTraceable for JSVal {
    fn trace(&self, trc: *mut JSTracer) {
        trace_jsval(trc, "val", *self);
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an no_jsmanaged_fields type)
impl<T: JSTraceable> JSTraceable for Vec<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an no_jsmanaged_fields type)
impl<T: JSTraceable + 'static> JSTraceable for SmallVec1<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
    }
}

impl<T: JSTraceable> JSTraceable for Option<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        self.as_ref().map(|e| e.trace(trc));
    }
}

impl<K,V,S,H> JSTraceable for HashMap<K, V, H> where K: Eq + Hash<S> + JSTraceable,
                                                     V: JSTraceable,
                                                     H: Hasher<S> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.val0().trace(trc);
            e.val1().trace(trc);
        }
    }
}

impl<A: JSTraceable, B: JSTraceable> JSTraceable for (A, B) {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        let (ref a, ref b) = *self;
        a.trace(trc);
        b.trace(trc);
    }
}


no_jsmanaged_fields!(bool, f32, f64, String, Url)
no_jsmanaged_fields!(uint, u8, u16, u32, u64)
no_jsmanaged_fields!(int, i8, i16, i32, i64)
no_jsmanaged_fields!(Sender<T>)
no_jsmanaged_fields!(Receiver<T>)
no_jsmanaged_fields!(Rect<T>)
no_jsmanaged_fields!(ImageCacheTask, ScriptControlChan)
no_jsmanaged_fields!(Atom, Namespace, Timer)
no_jsmanaged_fields!(Trusted<T>)
no_jsmanaged_fields!(PropertyDeclarationBlock)
// These three are interdependent, if you plan to put jsmanaged data
// in one of these make sure it is propagated properly to containing structs
no_jsmanaged_fields!(SubpageId, WindowSizeData, PipelineId)
no_jsmanaged_fields!(QuirksMode)
no_jsmanaged_fields!(Cx)
no_jsmanaged_fields!(ResponseHeaderCollection, RequestHeaderCollection, Method)
no_jsmanaged_fields!(ConstellationChan)
no_jsmanaged_fields!(LayoutChan)
no_jsmanaged_fields!(WindowProxyHandler)
no_jsmanaged_fields!(UntrustedNodeAddress)
no_jsmanaged_fields!(LengthOrPercentageOrAuto)

impl JSTraceable for Box<ScriptChan+Send> {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

impl<'a> JSTraceable for &'a str {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl<A,B> JSTraceable for fn(A) -> B {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<ScriptListener+'static> {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<LayoutRPC+'static> {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for TrustedNodeAddress {
    fn trace(&self, s: *mut JSTracer) {
        let TrustedNodeAddress(addr) = *self;
        let node = addr as *const Node;
        unsafe {
            JS::from_raw(node).trace(s)
        }
    }
}

/// Holds a set of vectors that need to be rooted
pub struct RootedCollectionSet {
    set: RefCell<Vec<HashSet<*const ()>>>
}

/// TLV Holds a set of vectors that need to be rooted
local_data_key!(pub RootedCollections: RootedCollectionSet)

enum CollectionType {
    DOMObjects,
    JSVals,
    JSObjects,
}


impl RootedCollectionSet {
    /// Called from script_task
    pub fn initialize() {
        assert!(RootedCollections.get().is_none());
        RootedCollections.replace(Some(RootedCollectionSet {
            set: RefCell::new(vec!(HashSet::new(), HashSet::new(), HashSet::new()))
        }));
    }

    fn remove<T: VecRootableType>(collection: &mut RootedVec<T>) {
        let type_ = VecRootableType::tag(None::<T>);
        let collections = RootedCollections.get();
        let mut collections = collections.as_ref().unwrap().set.borrow_mut();
        assert!((*collections)[type_ as uint].remove(&(collection as *mut _ as *const _)));
    }

    fn add<T: VecRootableType>(collection: &RootedVec<T>) {
        let type_ = VecRootableType::tag(None::<T>);
        let collections = RootedCollections.get();
        let mut collections = collections.as_ref().unwrap().set.borrow_mut();
        (*collections)[type_ as uint].insert(collection as *const _ as *const _);
    }

    unsafe fn trace(&self, tracer: *mut JSTracer) {
        fn trace_collection_type<T: JSTraceable>(tracer: *mut JSTracer,
                                                 collections: *const HashSet<*const RootedVec<T>>) {
            unsafe {
                for collection in (*collections).iter() {
                    let _ = (**collection).trace(tracer);
                }
            }
        }

        let collections = self.set.borrow();

        let dom_collections = &(*collections)[DOMObjects as uint] as *const _ as *const HashSet<*const RootedVec<*const Reflector>>;
        for dom_collection in (*dom_collections).iter() {
            for reflector in (**dom_collection).iter() {
                trace_reflector(tracer, "", &**reflector);
            }
        }

        trace_collection_type(tracer,
                              &(*collections)[JSVals as uint] as *const _ as *const HashSet<*const RootedVec<JSVal>>);
        trace_collection_type(tracer,
                              &(*collections)[JSObjects as uint] as *const _ as *const HashSet<*const RootedVec<*mut JSObject>>);
    }
}


/// Trait implemented by all types that can be used with RootedVec
trait VecRootableType {
    /// Return the type tag used to determine how to trace RootedVec
    fn tag(_a: Option<Self>) -> CollectionType;
}

impl<T: Reflectable> VecRootableType for JS<T> {
    fn tag(_a: Option<JS<T>>) -> CollectionType { DOMObjects }
}

impl VecRootableType for JSVal {
    fn tag(_a: Option<JSVal>) -> CollectionType { JSVals }
}

impl VecRootableType for *mut JSObject {
    fn tag(_a: Option<*mut JSObject>) -> CollectionType { JSObjects }
}

/// A vector of items that are rooted for the lifetime
/// of this struct
#[allow(unrooted_must_root)]
#[jstraceable]
pub struct RootedVec<T> {
    v: Vec<T>
}


impl<T: VecRootableType> RootedVec<T> {
    /// Create a vector of items of type T that is rooted for
    /// the lifetime of this struct
    pub fn new() -> RootedVec<T> {
        let ret = RootedVec::<T> { v: vec!() };
        unsafe {
            RootedCollectionSet::add::<T>(&*(return_address() as *const _));
        }
        ret
    }

}

#[unsafe_destructor]
impl<T: VecRootableType> Drop for RootedVec<T> {
    fn drop(&mut self) {
        RootedCollectionSet::remove(self);
    }
}

impl<T> Deref<Vec<T>> for RootedVec<T> {
    fn deref(&self) -> &Vec<T> {
        &self.v
    }
}

impl<T> DerefMut<Vec<T>> for RootedVec<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.v
    }
}


/// SM Callback that traces the rooted collections
pub unsafe extern fn trace_collections(tracer: *mut JSTracer, data: *mut libc::c_void) {
    (*(data as *const RootedCollectionSet)).trace(tracer);
}

