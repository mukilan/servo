/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::FontFaceBinding::{
    FontFaceLoadStatus, FontFaceMethods,
};
use crate::dom::bindings::codegen::UnionTypes;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct FontFace {
    reflector: Reflector,
}

impl FontFace {
    pub fn new_inherited(_global: &GlobalScope, _can_gc: CanGc) -> Self {
        Self {
            reflector: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope, proto: Option<HandleObject>, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(global, can_gc)),
            global,
            proto,
            can_gc,
        )
    }
}

impl FontFaceMethods<crate::DomTypeHolder> for FontFace {
    fn Family(&self) -> DOMString {
        todo!("Family")
    }
    fn SetFamily(&self, _value: DOMString) {
        todo!("SetFamily")
    }
    fn Style(&self) -> DOMString {
        todo!("Style")
    }
    fn SetStyle(&self, _value: DOMString) {
        todo!("SetStyle")
    }
    fn Weight(&self) -> DOMString {
        todo!("Weight")
    }
    fn SetWeight(&self, _value: DOMString) {
        todo!("SetWeight")
    }
    fn Stretch(&self) -> DOMString {
        todo!("Stretch")
    }
    fn SetStretch(&self, _value: DOMString) {
        todo!("SetStretch")
    }
    fn UnicodeRange(&self) -> DOMString {
        todo!("UnicodeRange")
    }
    fn SetUnicodeRange(&self, _value: DOMString) {
        todo!("SetUnicodeRange")
    }
    fn FeatureSettings(&self) -> DOMString {
        todo!("FeatureSettings")
    }
    fn SetFeatureSettings(&self, _value: DOMString) {
        todo!("SetFeatureSettings")
    }
    fn VariationSettings(&self) -> DOMString {
        todo!("VariationSettings")
    }
    fn SetVariationSettings(&self, _value: DOMString) {
        todo!("SetVariationSettings")
    }
    fn Display(&self) -> DOMString {
        todo!("Display")
    }
    fn SetDisplay(&self, _value: DOMString) {
        todo!("SetDisplay")
    }
    fn AscentOverride(&self) -> DOMString {
        todo!("AscentOverride")
    }
    fn SetAscentOverride(&self, _value: DOMString) {
        todo!("SetAscentOverride")
    }
    fn DescentOverride(&self) -> DOMString {
        todo!("DescentOverride")
    }
    fn SetDescentOverride(&self, _value: DOMString) {
        todo!("SetDescentOverride")
    }
    fn LineGapOverride(&self) -> DOMString {
        todo!("LineGapOverride")
    }
    fn SetLineGapOverride(&self, _value: DOMString) {
        todo!("SetLineGapOverride")
    }
    fn Status(&self) -> FontFaceLoadStatus {
        todo!("Status")
    }
    fn Load(&self) -> Rc<Promise> {
        todo!("Load")
    }
    fn Loaded(&self) -> Rc<Promise> {
        todo!("Loaded")
    }
    fn Constructor(
        _global: &GlobalScope,
        _proto: Option<HandleObject>,
        _can_gc: CanGc,
        _family: DOMString,
        _source: UnionTypes::StringOrArrayBufferViewOrArrayBuffer,
        _descriptors: &crate::dom::bindings::codegen::Bindings::FontFaceBinding::FontFaceDescriptors,
    ) -> DomRoot<FontFace> {
        todo!("FontFace Constructor")
    }
}
