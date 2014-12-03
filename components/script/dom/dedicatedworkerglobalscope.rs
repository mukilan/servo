/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding::DedicatedWorkerGlobalScopeMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::DedicatedWorkerGlobalScopeDerived;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, WorkerGlobalScopeCast};
use dom::bindings::error::{ErrorResult, DataClone};
use dom::bindings::global;
use dom::bindings::js::{JSRef, Temporary, RootCollection};
use dom::bindings::refcounted::LiveDOMReferences;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::{EventTarget, EventTargetHelpers};
use dom::eventtarget::WorkerGlobalScopeTypeId;
use dom::messageevent::MessageEvent;
use dom::worker::{Worker, TrustedWorkerAddress};
use dom::workerglobalscope::DedicatedGlobalScope;
use dom::workerglobalscope::{WorkerGlobalScope, WorkerGlobalScopeHelpers};
use dom::xmlhttprequest::XMLHttpRequest;
use script_task::{ScriptTask, ScriptChan};
use script_task::{ScriptMsg, FromWorker,  DOMMessage, FireTimerMsg, XHRProgressMsg};
use script_task::{WorkerPostMessage, RefcountCleanup};
use script_task::StackRootTLS;

use servo_net::resource_task::{ResourceTask, load_whole_resource};
use servo_util::task::spawn_named_native;
use servo_util::task_state;
use servo_util::task_state::{SCRIPT, IN_WORKER};

use js::glue::JS_STRUCTURED_CLONE_VERSION;
use js::jsapi::{JSContext, JS_ReadStructuredClone, JS_WriteStructuredClone, JS_ClearPendingException};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::Cx;

use std::rc::Rc;
use std::ptr;
use url::Url;

#[deriving(Clone)]
#[jstraceable]
pub struct SendableWorkerScriptChan {
    sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
    worker: TrustedWorkerAddress,
}

impl ScriptChan for SendableWorkerScriptChan {
    fn send(&self, msg: ScriptMsg) {
        self.sender.send((self.worker.clone(), msg));
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        box SendableWorkerScriptChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        }
    }
}

struct AutoWorkerReset<'a> {
    workerscope: JSRef<'a, DedicatedWorkerGlobalScope>,
    old_worker: Option<TrustedWorkerAddress>,
}

impl<'a> AutoWorkerReset<'a> {
    fn new(workerscope: JSRef<'a, DedicatedWorkerGlobalScope>, worker: TrustedWorkerAddress) -> AutoWorkerReset<'a> {
        let reset = AutoWorkerReset {
            workerscope: workerscope,
            old_worker: workerscope.worker.borrow().clone()
        };
        *workerscope.worker.borrow_mut() = Some(worker);
        reset
    }
}

#[unsafe_destructor]
impl<'a> Drop for AutoWorkerReset<'a> {
    fn drop(&mut self) {
        *self.workerscope.worker.borrow_mut() = self.old_worker.clone();
    }
}

#[dom_struct]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>,
    own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
    worker: DOMRefCell<Option<TrustedWorkerAddress>>,
    /// Sender to the parent thread.
    parent_sender: Box<ScriptChan+Send>,
}

impl DedicatedWorkerGlobalScope {
    fn new_inherited(worker_url: Url,
                         cx: Rc<Cx>,
                         resource_task: ResourceTask,
                         parent_sender: Box<ScriptChan+Send>,
                         own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
                         receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>)
                         -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                DedicatedGlobalScope, worker_url, cx, resource_task),
            receiver: receiver,
            own_sender: own_sender,
            parent_sender: parent_sender,
            worker: DOMRefCell::new(None),
        }
    }

    pub fn new(worker_url: Url,
               cx: Rc<Cx>,
               resource_task: ResourceTask,
               parent_sender: Box<ScriptChan+Send>,
               own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
               receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>)
               -> Temporary<DedicatedWorkerGlobalScope> {
        let scope = box DedicatedWorkerGlobalScope::new_inherited(
            worker_url, cx.clone(), resource_task, parent_sender,
            own_sender, receiver);
        DedicatedWorkerGlobalScopeBinding::Wrap(cx.ptr, scope)
    }
}

impl DedicatedWorkerGlobalScope {
    pub fn run_worker_scope(worker_url: Url,
                            worker: TrustedWorkerAddress,
                            resource_task: ResourceTask,
                            parent_sender: Box<ScriptChan+Send>,
                            own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
                            receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>) {
        spawn_named_native(format!("WebWorker for {}", worker_url.serialize()), proc() {

            task_state::initialize(SCRIPT | IN_WORKER);

            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);

            let (url, source) = match load_whole_resource(&resource_task, worker_url.clone()) {
                Err(_) => {
                    println!("error loading script {}", worker_url.serialize());
                    return;
                }
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            let (_js_runtime, js_context) = ScriptTask::new_rt_and_cx();
            let global = DedicatedWorkerGlobalScope::new(
                worker_url, js_context.clone(), resource_task,
                parent_sender, own_sender, receiver).root();

            {
                let _ar = AutoWorkerReset::new(*global, worker);

                match js_context.evaluate_script(
                    global.reflector().get_jsobject(), source, url.serialize(), 1) {
                    Ok(_) => (),
                    Err(_) => println!("evaluate_script failed")
                }
            }

            loop {
                match global.receiver.recv_opt() {
                    Ok((linked_worker, msg)) => {
                        let _ar = AutoWorkerReset::new(*global, linked_worker);
                        global.handle_event(msg);
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

pub trait DedicatedWorkerGlobalScopeHelpers {
    fn script_chan(self) -> Box<ScriptChan+Send>;
}

impl<'a> DedicatedWorkerGlobalScopeHelpers for JSRef<'a, DedicatedWorkerGlobalScope> {
    fn script_chan(self) -> Box<ScriptChan+Send> {
        box SendableWorkerScriptChan {
            sender: self.own_sender.clone(),
            worker: self.worker.borrow().as_ref().unwrap().clone(),
        }
    }
}

trait PrivateDedicatedWorkerGlobalScopeHelpers {
    fn handle_event(self, msg: ScriptMsg);
}

impl<'a> PrivateDedicatedWorkerGlobalScopeHelpers for JSRef<'a, DedicatedWorkerGlobalScope> {
    fn handle_event(self, msg: ScriptMsg) {
        match msg {
            DOMMessage(data, nbytes) => {
                let mut message = UndefinedValue();
                let scope: JSRef<WorkerGlobalScope> = WorkerGlobalScopeCast::from_ref(self);
                unsafe {
                    assert!(JS_ReadStructuredClone(
                        scope.get_cx(), data as *const u64, nbytes,
                        JS_STRUCTURED_CLONE_VERSION, &mut message,
                        ptr::null(), ptr::null_mut()) != 0);
                }

                let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
                MessageEvent::dispatch_jsval(target, &global::Worker(scope), message);
            },
            XHRProgressMsg(addr, progress) => {
                XMLHttpRequest::handle_progress(addr, progress)
            },
            WorkerPostMessage(addr, data, nbytes) => {
                Worker::handle_message(addr, data, nbytes);
            },
            RefcountCleanup(addr) => {
                let scope: JSRef<WorkerGlobalScope> = WorkerGlobalScopeCast::from_ref(self);
                LiveDOMReferences::cleanup(scope.get_cx(), addr);
            }
            FireTimerMsg(FromWorker, timer_id) => {
                let scope: JSRef<WorkerGlobalScope> = WorkerGlobalScopeCast::from_ref(self);
                scope.handle_fire_timer(timer_id);
            }
            _ => panic!("Unexpected message"),
        }
    }
}

impl<'a> DedicatedWorkerGlobalScopeMethods for JSRef<'a, DedicatedWorkerGlobalScope> {
    fn PostMessage(self, cx: *mut JSContext, message: JSVal) -> ErrorResult {
        let mut data = ptr::null_mut();
        let mut nbytes = 0;
        let result = unsafe {
            JS_WriteStructuredClone(cx, message, &mut data, &mut nbytes,
                                    ptr::null(), ptr::null_mut())
        };
        if result == 0 {
            unsafe { JS_ClearPendingException(cx); }
            return Err(DataClone);
        }

        let worker = self.worker.borrow().as_ref().unwrap().clone();
        self.parent_sender.send(WorkerPostMessage(worker, data, nbytes));
        Ok(())
    }

    event_handler!(message, GetOnmessage, SetOnmessage)
}

impl Reflectable for DedicatedWorkerGlobalScope {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.workerglobalscope.reflector()
    }
}

impl DedicatedWorkerGlobalScopeDerived for EventTarget {
    fn is_dedicatedworkerglobalscope(&self) -> bool {
        match *self.type_id() {
            WorkerGlobalScopeTypeId(DedicatedGlobalScope) => true,
            _ => false
        }
    }
}
