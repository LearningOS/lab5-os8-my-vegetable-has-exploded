use super::sys_gettid;
use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::resources::*;
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use log::*;
use alloc::vec;

pub fn sys_sleep(ms: usize) -> isize {
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}

// LAB5 HINT: you might need to maintain data structures used for deadlock detection
// during sys_mutex_* and sys_semaphore_* syscalls
pub fn sys_mutex_create(blocking: bool) -> isize {
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        process_inner.mutex_resources.add_resource(id, 1);
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        let mutex_id = process_inner.mutex_list.len() - 1;
        process_inner.mutex_resources.add_resource(mutex_id, 1);
        mutex_id as isize
    }
}

// LAB5 HINT: Return -0xDEAD if deadlock is detected
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let tid = sys_gettid() as usize;
	debug!("lock tid: {}", tid);
    if process_inner.is_deadlock_detect_enabled()
        && process_inner.mutex_resources.deadlock_detect(
            tid,
            vec![ResourceElement {
                resource_id: mutex_id,
                resource_num: 1,
            }],
        )
    {
        return -0xDEAD;
    }
    process_inner.mutex_resources.need_resource(
        tid,
        vec![ResourceElement {
            resource_id: mutex_id,
            resource_num: 1,
        }],
    );
    drop(process_inner);
    drop(process);
    mutex.lock();
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.mutex_resources.alloc_resource(tid);
    drop(process_inner);
    drop(process);
    0
}

pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let tid = sys_gettid() as usize;
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    process_inner.mutex_resources.dealloc_resource(
        tid,
        vec![ResourceElement {
            resource_id: mutex_id,
            resource_num: 1,
        }],
    );
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}

pub fn sys_semaphore_create(res_count: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    process_inner
        .semaphore_resources
        .add_resource(id, res_count);
    drop(process_inner);
    drop(process);
    id as isize
}

pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let tid = sys_gettid() as usize;
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
	debug!("sema up tid: {} sem_id: {}", tid, sem_id);
    process_inner.semaphore_resources.dealloc_resource(
        tid,
        vec![ResourceElement {
            resource_id: sem_id,
            resource_num: 1,
        }],
    );
    drop(process_inner);
    sem.up();
    0
}

// LAB5 HINT: Return -0xDEAD if deadlock is detected
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let tid = sys_gettid() as usize;
	debug!("sema down tid: {} sem_id: {}", tid, sem_id);
    if process_inner.is_deadlock_detect_enabled()
        && process_inner.semaphore_resources.deadlock_detect(
            tid,
            vec![ResourceElement {
                resource_id: sem_id,
                resource_num: 1,
            }],
        )
    {
        return -0xDEAD;
    }
    process_inner.semaphore_resources.need_resource(
        tid,
        vec![ResourceElement {
            resource_id: sem_id,
            resource_num: 1,
        }],
    );
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    sem.down();
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.semaphore_resources.alloc_resource(tid);
    drop(process_inner);
    drop(process);
    0
}

pub fn sys_condvar_create(_arg: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}

pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}

pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}

// LAB5 YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let ret = match _enabled {
        0 => {
            process_inner.disable_deadlock_detect();
            0
        }
        1 => {
            process_inner.enable_deadlock_detect();
            0
        }
        _ => -1,
    };
    drop(process_inner);
    return ret;
}
