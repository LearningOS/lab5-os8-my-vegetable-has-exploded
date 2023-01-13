use alloc::vec;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceElement {
    pub resource_id: usize,
    pub resource_num: usize,
}

pub struct Resources {
    pub thread_num: usize,
    pub allocation: Vec<Vec<ResourceElement>>,
    pub need: Vec<Vec<ResourceElement>>,
    pub avaliable: Vec<ResourceElement>,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            thread_num: 0 as usize,
            allocation: Vec::new(),
            need: Vec::new(),
            avaliable: Vec::new(),
        }
    }

    pub fn deadlock_detect(&self, tid: usize, apply: Vec<ResourceElement>) -> bool {
        let mut work = self.avaliable.clone();
        let mut finish = vec![false; self.thread_num];
        let mut flag = true;
        let mut need = self.need.clone();
        for resource_element in apply.iter() {
            let rid = resource_element.resource_id;
            let rnum = resource_element.resource_num;
            match need[tid]
                .iter_mut()
                .enumerate()
                .find(|x| x.1.resource_id == rid)
            {
                Some((index, resource_element)) => {
                    resource_element.resource_num += rnum;
                }
                None => {
                    need[tid].push(ResourceElement {
                        resource_id: rid,
                        resource_num: rnum,
                    });
                }
            }
        }
        debug!(
            "need: {:?} alloc: {:?} work: {:?}",
            need, self.allocation, work
        );
        while flag {
            flag = false;
            for i in 0..self.thread_num {
                debug!("test tid: {}", i);
                if !finish[i] {
                    let mut j = 0;
                    while j < need[i].len() {
                        debug!("test j : {:?}", need[i][j]);
                        let rid = need[i][j].resource_id;
                        let rnum = work
                            .iter()
                            .find(|x| x.resource_id == rid)
                            .unwrap()
                            .resource_num;
                        if need[i][j].resource_num > rnum {
                            debug!(
                                "need[i][j].resource_num: {} rnum: {}",
                                need[i][j].resource_num, rnum
                            );
                            break;
                        }
                        j += 1;
                    }
                    if j == need[i].len() {
                        for j in 0..self.allocation[i].len() {
                            let rid = self.allocation[i][j].resource_id;
                            let rnum = self.allocation[i][j].resource_num;
                            work[rid].resource_num += rnum;
                        }
                        finish[i] = true;
                        flag = true;
                    }
                }
            }
        }
        for i in 0..self.thread_num {
            if !finish[i] {
                return true;
            }
        }
        return false;
    }

    pub fn add_resource(&mut self, resource_id: usize, resource_num: usize) {
        let resource = ResourceElement {
            resource_id: resource_id,
            resource_num: resource_num,
        };
        self.avaliable.push(resource);
    }

    pub fn create_thread(&mut self, thread_id: usize) {
        for i in self.thread_num..thread_id + 1 {
            self.thread_num = thread_id + 1;
            self.allocation.push(Vec::new());
            self.need.push(Vec::new());
        }
    }

    pub fn release_thread(&mut self, thread_id: usize) {
        self.need[thread_id].clear();
        for resource_element in self.allocation[thread_id].iter() {
            let rid = resource_element.resource_id;
            let rnum = resource_element.resource_num;
            self.avaliable[rid].resource_num += rnum;
        }
    }

    pub fn alloc_resource(&mut self, thread_id: usize) {
        for resource_element in self.need[thread_id].iter() {
            let rid = resource_element.resource_id;
            let rnum = resource_element.resource_num;
            self.avaliable[rid].resource_num -= rnum;
            match self.allocation[thread_id]
                .iter_mut()
                .enumerate()
                .find(|x| x.1.resource_id == rid)
            {
                Some((index, resource_element)) => {
                    resource_element.resource_num += rnum;
                }
                None => {
                    self.allocation[thread_id].push(ResourceElement {
                        resource_id: rid,
                        resource_num: rnum,
                    });
                }
            }
        }
        self.need[thread_id].clear();
    }

    pub fn need_resource(&mut self, thread_id: usize, resources: Vec<ResourceElement>) {
        for resource_element in resources.iter() {
            let rid = resource_element.resource_id;
            let rnum = resource_element.resource_num;
            match self.need[thread_id]
                .iter_mut()
                .enumerate()
                .find(|x| x.1.resource_id == rid)
            {
                Some((index, resource_element)) => {
                    resource_element.resource_num += rnum;
                }
                None => {
                    self.need[thread_id].push(ResourceElement {
                        resource_id: rid,
                        resource_num: rnum,
                    });
                }
            }
        }
    }

    pub fn dealloc_resource(&mut self, thread_id: usize, resources: Vec<ResourceElement>) {
        for resource_element in resources.iter() {
            let rid = resource_element.resource_id;
            let rnum = resource_element.resource_num;
            self.allocation[thread_id]
                .iter_mut()
                .find(|x| x.resource_id == rid)
                .map(|x| x.resource_num -= rnum);
            self.avaliable[rid].resource_num += rnum;
        }
    }
}
