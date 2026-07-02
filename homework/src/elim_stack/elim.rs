use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::Ordering;
use std::{fs::write, thread};

use crossbeam_epoch::{Guard, Owned, Shared};

use super::base::{ELIM_DELAY, ElimStack, Stack, get_random_elim_index};

impl<T, S: Stack<T>> Stack<T> for ElimStack<T, S> {
    type PushReq = S::PushReq;

    fn try_push(
        &self,
        req: Owned<Self::PushReq>,
        guard: &Guard,
    ) -> Result<(), Owned<Self::PushReq>> {
        let Err(req) = self.inner.try_push(req, guard) else {
            return Ok(());
        };

        let index = get_random_elim_index();
        let slot_ref = unsafe { self.slots.get_unchecked(index) };
        let slot = slot_ref.load(Ordering::Acquire, guard);

        match slot.tag() {
            0 => {
                let req_shared = req.into_shared(guard);
                // CAS to offer our push request with tag 1
                match slot_ref.compare_exchange(slot, req_shared.with_tag(1), Ordering::Release, Ordering::Relaxed, guard) {
                    Ok(_) => {
                        thread::sleep(ELIM_DELAY);
                        let current_slot = slot_ref.load(Ordering::Acquire, guard);
                        
                        if current_slot == req_shared.with_tag(1) {
                            // Nobody matched with us. Attempt to reclaim our node.
                            match slot_ref.compare_exchange(req_shared.with_tag(1), Shared::null(), Ordering::Release, Ordering::Relaxed, guard) {
                                Ok(_) => Err(unsafe { req_shared.into_owned() }),
                                Err(_) => Ok(()), // A pop thread matched at the very last millisecond (tag became 3)
                            }
                        } else {
                            // The slot changed. It's either our request marked as acknowledged (tag 3)
                            // or it was already cleared and reused by another thread.
                            // Only clear it if it explicitly belongs to our transaction.
                            if current_slot == req_shared.with_tag(3) {
                                let _ = slot_ref.compare_exchange(req_shared.with_tag(3), Shared::null(), Ordering::Release, Ordering::Relaxed, guard);
                            }
                            Ok(())
                        }
                    }
                    Err(_) => Err(unsafe { req_shared.into_owned() }),
                }
            },
            2 => {
                // A pop thread is waiting with an empty slot reservation (tag 2)
                let req_shared = req.into_shared(guard);
                // Fulfill their pop request by changing tag 2 to tag 3 with our node
                match slot_ref.compare_exchange(slot, req_shared.with_tag(3), Ordering::Release, Ordering::Relaxed, guard) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(unsafe { req_shared.into_owned() }),
                }
            },
            _ => Err(req), // Collision with an active exchange, retry later
        }
    }

    fn try_pop(&self, guard: &Guard) -> Result<Option<T>, ()> {
        if let Ok(result) = self.inner.try_pop(guard) {
            return Ok(result);
        }

        let index = get_random_elim_index();
        let slot_ref = unsafe { self.slots.get_unchecked(index) };
        let slot = slot_ref.load(Ordering::Acquire, guard);

        match slot.tag() {
            0 => {
                match slot_ref.compare_exchange(Shared::null(), Shared::null().with_tag(2), Ordering::Release, Ordering::Relaxed, guard) {
                    Ok(_) => {
                        thread::sleep(ELIM_DELAY);
                        let current_slot = slot_ref.load(Ordering::Acquire, guard);
                        
                        if current_slot.tag() == 3 {
                            let data = ManuallyDrop::into_inner(unsafe {
                                ptr::read(&**current_slot.as_ref().unwrap())
                            });
                            
                            let _ = slot_ref.compare_exchange(
                                current_slot,
                                Shared::null(),
                                Ordering::Release,
                                Ordering::Relaxed,
                                guard,
                            );
                            
                            unsafe { guard.defer_destroy(current_slot) };
                            Ok(Some(data))
                        } else {
                            match slot_ref.compare_exchange(Shared::null().with_tag(2), Shared::null(), Ordering::Release, Ordering::Relaxed, guard) {
                                Ok(_) => Err(()), 
                                Err(e) => {
                                    let actual_slot = e.current;
                                    
                                    let data = ManuallyDrop::into_inner(unsafe {
                                        ptr::read(&**actual_slot.as_ref().unwrap())
                                    });
                                    
                                    let _ = slot_ref.compare_exchange(actual_slot, Shared::null(), Ordering::Release, Ordering::Relaxed, guard);
                                    unsafe { guard.defer_destroy(actual_slot) };
                                    Ok(Some(data))
                                }
                            }
                        }
                    }
                    Err(_) => Err(()), 
                }
            }
            1 => {
                let target = slot.with_tag(3);
                match slot_ref.compare_exchange(slot, target, Ordering::Release, Ordering::Relaxed, guard) {
                    Ok(_) => {
                        let data = ManuallyDrop::into_inner(unsafe {
                            ptr::read(&**slot.as_ref().unwrap())
                        });
                        
                        unsafe { guard.defer_destroy(slot) };
                        Ok(Some(data))
                    }
                    Err(_) => Err(()), 
                }
            }
            _ => Err(()), 
        }
    }

    fn is_empty(&self, guard: &Guard) -> bool {
        self.inner.is_empty(guard)
    }
}
