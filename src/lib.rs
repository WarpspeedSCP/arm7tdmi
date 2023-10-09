
mod cpu;

/**
 * This module declares enums encoding ARM7TDI instruction and data formats, as well as supporting data such as general purpose registers, flag registers, interrupt vector tables and other processor internal data structures.
 */
mod isa;

/**
 * This module defines input and output for the ARM7TDI processor, including traits defining behavior for all ports available to it (including memory access modes and interprocessor communication protocols)
 */
mod io;


/**
 * This module defines the interface of the ARM7TDI processor's execution unit, and provides a trait through which all execution related operations (the instruction pipeline, interrupt execution and interfacing with the registers, ALU and data ports of the processor) are defined.
 */
mod eu;

pub trait Wrappable where Self: Sized {
    fn wrap_ok<E>(self) -> Result<Self, E> {
        Ok(self)
    }
    fn wrap_err<T>(self) -> Result<T, Self> {
        Err(self)
    }

    fn wrap_some(self) -> Option<Self> {
        Some(self)
    }
}

impl <T> Wrappable for T where T: Sized {}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
