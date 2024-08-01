//! SysTick-based time driver.
//! modified from https://github.com/ch32-rs/ch32-hal/blob/main/src/embassy/time_driver_systick.rs

use core::cell::Cell;
use core::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use core::{mem, ptr};

use critical_section::{CriticalSection, Mutex};
use embassy_time_driver::{AlarmHandle, Driver};

pub const ALARM_COUNT: usize = 1;

struct AlarmState {
    timestamp: Cell<u64>,

    // This is really a Option<(fn(*mut ()), *mut ())>
    // but fn pointers aren't allowed in const yet
    callback: Cell<*const ()>,
    ctx: Cell<*mut ()>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
            callback: Cell::new(ptr::null()),
            ctx: Cell::new(ptr::null_mut()),
        }
    }
}

pub struct SystickDriver {
    alarm_count: AtomicU8,
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
    // period: AtomicU32,
    tick_low: AtomicU32,
    tick_high: AtomicU32,
}

const ALARM_STATE_NEW: AlarmState = AlarmState::new();
embassy_time_driver::time_driver_impl!(static DRIVER: SystickDriver = SystickDriver {
    // period: AtomicU32::new(1), // avoid div by zero
    alarm_count: AtomicU8::new(0),
    alarms: Mutex::new([ALARM_STATE_NEW; ALARM_COUNT]),
    tick_low: AtomicU32::new(1),
    tick_high: AtomicU32::new(0),
});

impl SystickDriver {
    fn init(&'static self) {
        self.tick_low.store(1, Ordering::Relaxed);
        self.tick_high.store(0, Ordering::Relaxed);
    }

    #[inline(always)]
    fn on_interrupt(&self) {
        if self.tick_low.load(Ordering::Relaxed) == u32::MAX {
            self.tick_low.store(0, Ordering::Relaxed);
            // thembv6m only supported store and load
            // self.tick_high.fetch_add(1, Ordering::Relaxed);
            
            
            self.tick_high.store(
                self.tick_high.load(Ordering::Relaxed) + 1,
                Ordering::Relaxed);
        }
        // self.tick_low.fetch_add(1, Ordering::Relaxed);

        self.tick_low.store(
            self.tick_low.load(Ordering::Relaxed) + 1,
            Ordering::Relaxed);
        

        critical_section::with(|cs| {
            let timestamp = self.alarms.borrow(cs)[0].timestamp.get();
            if timestamp <= self.now() + 1 {
                self.trigger_alarm(cs);
            }
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let alarm = &self.alarms.borrow(cs)[0];
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.

        // safety:
        // - we can ignore the possiblity of `f` being unset (null) because of the safety contract of `allocate_alarm`.
        // - other than that we only store valid function pointers into alarm.callback
        let f: fn(*mut ()) = unsafe { mem::transmute(alarm.callback.get()) };
        f(alarm.ctx.get());
    }

    fn get_alarm<'a>(&'a self, cs: CriticalSection<'a>, alarm: AlarmHandle) -> &'a AlarmState {
        // safety: we're allowed to assume the AlarmState is created by us, and
        // we never create one that's out of bounds.
        unsafe { self.alarms.borrow(cs).get_unchecked(alarm.id() as usize) }
    }
}

impl Driver for SystickDriver {
    fn now(&self) -> u64 {
        let low = self.tick_low.load(Ordering::Relaxed);
        let high = self.tick_high.load(Ordering::Relaxed);
        ((high as u64) << 32) | (low as u64)
    }
    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        let old_count = self.alarm_count.load(Ordering::Acquire);
        let id = if old_count < ALARM_COUNT as u8 {
            self.alarm_count.store(old_count + 1, Ordering::Release);
            Some(old_count)
        } else {
            None
        };

        match id {
            Some(id) => Some(AlarmHandle::new(id)),
            None => None,
        }
    }
    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        critical_section::with(|cs| {
            let alarm = self.get_alarm(cs, alarm);

            alarm.callback.set(callback as *const ());
            alarm.ctx.set(ctx);
        })
    }
    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        critical_section::with(|cs| {
            let alarm = self.get_alarm(cs, alarm);
            alarm.timestamp.set(timestamp);
            if timestamp <= self.now() {
                // If alarm timestamp has passed the alarm will not fire.
                // Disarm the alarm and return `false` to indicate that.

                alarm.timestamp.set(u64::MAX);
                return false;
            }
            true
        })
    }
}

pub(crate) fn init() {
    DRIVER.init();
}

pub(crate) fn on_interrupt() {
    DRIVER.on_interrupt();
}

// pub fn now() -> u64 {
//     DRIVER.now()
// }