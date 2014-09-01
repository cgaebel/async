use cloq::{CloQ,CloSet,CloB,StopCondition,Stop,KeepGoing};

struct Scheduler {
  q: CloQ
}

impl Scheduler {
  fn new() -> Scheduler {
    Scheduler {
      q: CloQ::new(),
    }
  }
}

local_data_key!(task_local_scheduler: Scheduler)

#[inline]
fn with_scheduler<T>(f: |&mut Scheduler| -> T) -> T {
  let mut s =
    task_local_scheduler
      .replace(None)
      .unwrap_or_else(|| Scheduler::new());

  let r = f(&mut s);
  task_local_scheduler.replace(Some(s));
  r
}

/// Returns `true` if anything was run, `false` if the scheduler is empty.
#[inline]
pub fn tick(bucket: &mut CloB) -> bool {
  with_scheduler(|s: &mut Scheduler| {
    bucket.fill_from(&mut s.q);
  });

  match bucket.try_pop_and_run() {
    None            => false,
    Some(Stop)      => true,
    Some(KeepGoing) => {
      schedule_bucket(bucket);
      true
    }
  }
}

/// Pumps the scheduler until it's out of tasks.
pub fn tick_until_empty() {
  let mut bucket = CloB::new();
  while tick(&mut bucket) {}
}

/// Schedules a function to be run once.
#[inline]
pub fn schedule<F: FnOnce<(), ()>>(f: F) {
  let mut my_f = Some(f);
  with_scheduler(|s: &mut Scheduler| {
    s.q.push_fnonce(my_f.take().unwrap())
  })
}

/// Schedules a function to be run until it returns `Stop`.
///
/// Each time the function returns `KeepGoing`, other closures in the scheduler
/// queue will have a chance to run.
#[inline]
pub fn schedule_fn<F: Fn<(), StopCondition>>(f: F) {
  let mut my_f = Some(f);
  with_scheduler(|s: &mut Scheduler| {
    s.q.push_fn(my_f.take().unwrap())
  })
}

#[inline]
pub fn schedule_fnmut<F: FnMut<(), StopCondition>>(f: F) {
  let mut my_f = Some(f);
  with_scheduler(|s: &mut Scheduler| {
    s.q.push_fnmut(my_f.take().unwrap())
  })
}

#[inline]
pub fn schedule_set(s: CloSet) {
  let mut my_s = Some(s);
  with_scheduler(|s: &mut Scheduler| {
    s.q.push_set(my_s.take().unwrap())
  })
}

#[inline]
pub fn schedule_queue(q: CloQ) {
  let mut my_q = Some(q);
  with_scheduler(|s: &mut Scheduler| {
    s.q.push_q(my_q.take().unwrap())
  })
}

#[inline]
pub fn schedule_bucket(b: &mut CloB) {
  let mut my_b = Some(b);
  with_scheduler(|s: &mut Scheduler| {
    s.q.push_b(my_b.take().unwrap())
  })
}
