use pete::{error::Result, x86::DebugRegister, Tracee};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreakpointSlot {
    Slot0,
    Slot1,
    Slot2,
    Slot3,
}

impl BreakpointSlot {
    fn enable_local_mask(&self) -> u64 {
        use BreakpointSlot::*;

        match self {
            Slot0 => 1 << 0,
            Slot1 => 1 << 2,
            Slot2 => 1 << 4,
            Slot3 => 1 << 6,
        }
    }
}

impl From<BreakpointSlot> for DebugRegister {
    fn from(slot: BreakpointSlot) -> Self {
        match slot {
            BreakpointSlot::Slot0 => DebugRegister::Dr0,
            BreakpointSlot::Slot1 => DebugRegister::Dr1,
            BreakpointSlot::Slot2 => DebugRegister::Dr2,
            BreakpointSlot::Slot3 => DebugRegister::Dr3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DebugFlag {
    Trap,
    Resume,
}

impl From<DebugFlag> for u64 {
    fn from(flag: DebugFlag) -> Self {
        match flag {
            DebugFlag::Trap => 0x100,
            DebugFlag::Resume => 0x10000,
        }
    }
}

pub trait HardwareDebug: seal::Sealed {
    fn set_breakpoint(&mut self, slot: BreakpointSlot, va: u64) -> Result<()>;

    fn clear_breakpoint(&mut self, slot: BreakpointSlot) -> Result<()>;

    fn set_debug_flag(&mut self, flag: DebugFlag, set: bool) -> Result<()>;
}

impl HardwareDebug for Tracee {
    fn set_breakpoint(&mut self, slot: BreakpointSlot, va: u64) -> Result<()> {
        let dr = slot.into();
        self.set_debug_register(dr, va)?;

        let enable_local = slot.enable_local_mask();
        let dr7: u64 = DR7_RESERVED | enable_local;

        self.set_debug_register(DebugRegister::Dr7, dr7)?;

        Ok(())
    }

    fn clear_breakpoint(&mut self, slot: BreakpointSlot) -> Result<()> {
        let dr = slot.into();
        self.set_debug_register(dr, 0)?;

        self.set_debug_register(DebugRegister::Dr7, DR7_RESERVED)?;

        Ok(())
    }

    fn set_debug_flag(&mut self, flag: DebugFlag, set: bool) -> Result<()> {
        let mut regs = self.registers()?;

        let mask  = u64::from(flag);

        regs.eflags &= !mask;

        if set {
            regs.eflags |= mask
        }

        self.set_registers(regs)?;

        Ok(())
    }
}

const DR7_RESERVED: u64 = 0x100;

mod seal {
    use pete::Tracee;

    pub trait Sealed {}

    impl Sealed for Tracee {}
}
