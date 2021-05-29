use crate::*;

use super::InterruptContext;
use super::InterruptType;

/// Interrupt Handler
pub fn interrupt_handler(interrupt_context: InterruptContext) -> usize
{
    kdebugln!(Interrupts, "{}", interrupt_context);

    // Check if there is a process running
    if let Some(proc) = process::scheduler::current_process()
    {
        if (interrupt_context.get_status() >> 11) & 3 == 0
        {
            proc.program_counter = interrupt_context.instruction_address();
        }
    }

    match interrupt_context.get_cause()
    {
        InterruptType::MachineExternalInterrupt =>
        {
            // Get the next external interrupt
            if let Some(interrupt) = unsafe { drivers::PLIC_DRIVER.next_interrupt() }
            {
                // Run the handler
                super::extint::external_interrupt_handler(interrupt, &interrupt_context);

                // Complete the interrupt
                unsafe { drivers::PLIC_DRIVER.complete(interrupt) }; 
            }
        },
        InterruptType::UserEnvironmentCall =>
        {
            let result =syscalls::handle_syscall(process::scheduler::current_process().unwrap(),
                                                    interrupt_context.get_frame_mut().regs[17],
                                                    interrupt_context.get_frame_mut().regs[10],
                                                    interrupt_context.get_frame_mut().regs[11],
                                                    interrupt_context.get_frame_mut().regs[12],
                                                    interrupt_context.get_frame_mut().regs[13],
                                                    interrupt_context.get_frame_mut().regs[14],
                                                    interrupt_context.get_frame_mut().regs[15],
                                                    interrupt_context.get_frame_mut().regs[16]);

            interrupt_context.get_frame_mut().regs[10] = result;

            return interrupt_context.instruction_address() + 4;
        },
        InterruptType::MachineTimerInterrupt =>
        {
            let schedule = process::scheduler::schedule_next();
            // Prepare the timer for the next tick
            unsafe { drivers::TIMER_DRIVER.trigger() }

            process::scheduler::schedule_jump(schedule);
        },
        default =>
        {
            kerrorln!("{}", interrupt_context);
            panic!("Unhandled Trap: {:?}", default);
        }
    }

    interrupt_context.instruction_address()
}