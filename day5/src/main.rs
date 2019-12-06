use std::convert::{TryFrom, TryInto};
use std::io::{stdin, stdout, Write};

use anyhow::{format_err, Context, Error, Result};
use log::debug;

fn string_to_vec(input: &str) -> Result<Tape> {
    let mut ret = Vec::new();
    for num in input.trim().split(",") {
        ret.push(num.parse()?);
    }
    Ok(Tape::new(&ret))
}

fn read_input(filename: &str) -> Result<Tape> {
    let data = std::fs::read_to_string(filename)?;

    string_to_vec(&data)
}

#[derive(Debug)]
enum OpCode {
    Add,
    Multiply,
    Input,
    Output,
    Terminate,
}

impl OpCode {
    fn argument_count(&self) -> usize {
        match self {
            OpCode::Add | OpCode::Multiply => 3,
            OpCode::Input | OpCode::Output => 1,
            OpCode::Terminate => 0,
        }
    }
}

#[derive(Clone)]
struct Tape {
    program: Vec<i64>,
}

impl Tape {
    fn new(program: &Vec<i64>) -> Self {
        Tape {
            program: program.clone(),
        }
    }

    fn get(&self, offset: usize) -> Option<i64> {
        self.program.get(offset).cloned()
    }

    fn set(&mut self, offset: usize, value: i64) -> Result<()> {
        if offset >= self.program.len() {
            return Err(format_err!("Attempted to set offset at a larger offset than the tape contains (attempted set of {}, length {})", offset, self.program.len()));
        }

        debug!("[SET] [{}] = {}", offset, value);

        self.program[offset] = value;

        Ok(())
    }
}

#[derive(Debug)]
enum FetchMode {
    Immediate,
    Position,
}

impl TryFrom<char> for FetchMode {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '0' => Ok(FetchMode::Position),
            '1' => Ok(FetchMode::Immediate),
            _ => Err(format_err!("Unknown mode {}", value)),
        }
    }
}

#[derive(Debug)]
struct Argument {
    mode: FetchMode,
    value: i64,
}

impl Argument {
    fn get(&self, tape: &Tape) -> Option<i64> {
        match self.mode {
            FetchMode::Immediate => Some(self.value),
            FetchMode::Position => tape.get(self.value as usize),
        }
    }
}

#[derive(Debug)]
struct Instruction {
    position: usize,
    opcode: OpCode,
    arguments: Vec<Argument>,
}

impl Instruction {
    fn new(tape: &Tape, offset: usize) -> Result<Self> {
        let code = format!(
            "{:0>2}",
            tape.get(offset)
                .ok_or(format_err!("No opcode found at offset {}", offset))?
        );

        let opcode: OpCode = code[code.len() - 2..code.len()]
            .try_into()
            .with_context(|| format!("Failed to parse opcode \"{}\"", code))?;

        let argument_count = opcode.argument_count();
        let mut arguments = Vec::new();
        for (i, c) in format!(
            "{:0>width$}",
            &code[..code.len() - 2],
            width = argument_count,
        )
        .chars()
        .rev()
        .enumerate()
        {
            let mode = if i == argument_count - 1 {
                // TODO(jsvana): fix this...
                match opcode {
                    OpCode::Output => FetchMode::Position,
                    _ => FetchMode::Immediate,
                }
            } else {
                c.try_into()
                    .with_context(|| format!("Failed to parse mode \"{}\"", c))?
            };

            arguments.push(Argument {
                mode,
                value: tape.get(offset + i + 1).unwrap(),
            })
        }

        let instruction = Instruction {
            position: offset,
            opcode,
            arguments,
        };

        instruction
            .validate()
            .context("Instruction failed validation")?;

        Ok(instruction)
    }

    fn validate(&self) -> Result<()> {
        let expected_argument_count = self.opcode.argument_count();

        if self.arguments.len() != expected_argument_count {
            return Err(format_err!(
                "Expected {} argument(s), got {}",
                expected_argument_count,
                self.arguments.len()
            ));
        }

        /*
        if let OpCode::Add | OpCode::Multiply | OpCode::Input = self.opcode {
            // This is fine because we've checked the argument count above
            if let FetchMode::Immediate = self
                .arguments
                .last()
                .ok_or(format_err!(
                    "Last argument not found for opcode {:?}",
                    self.opcode
                ))?
                .mode
            {
                return Err(format_err!(
                    "Non-immediate instruction passed to immediate-only instruction: {:?}",
                    self
                ));
            }
        }
        */

        Ok(())
    }

    fn get_argument_value(&self, tape: &Tape, index: usize) -> Result<i64> {
        self.arguments
            .get(index)
            .ok_or(format_err!(
                "Argument {} not found for opcode {:?}",
                index + 1,
                self.opcode
            ))?
            .get(tape)
            .ok_or(format_err!(
                "Argument {} for opcode {:?} is None",
                index + 1,
                self.opcode
            ))
    }

    fn run(&self, tape: &mut Tape) -> Result<InstructionResult> {
        debug!("{:?}", self);
        match self.opcode {
            OpCode::Add => {
                let arg1 = self.get_argument_value(tape, 0)?;
                let arg2 = self.get_argument_value(tape, 1)?;
                let result_offset = self.get_argument_value(tape, 2)?;
                let result = arg1 + arg2;

                debug!(
                    "[ADD] {} + {} = {}, [{}]",
                    arg1, arg2, result, result_offset
                );

                tape.set(result_offset as usize, result).with_context(|| {
                    format!(
                        "Failed to set multiplied value {} to tape index {}",
                        result, result_offset
                    )
                })?;

                Ok(InstructionResult::Continue)
            }
            OpCode::Multiply => {
                let arg1 = self.get_argument_value(tape, 0)?;
                let arg2 = self.get_argument_value(tape, 1)?;
                let result_offset = self.get_argument_value(tape, 2)?;

                let result = arg1 * arg2;

                debug!(
                    "[MUL] {} * {} = {}, [{}]",
                    arg1, arg2, result, result_offset
                );

                tape.set(result_offset as usize, result).with_context(|| {
                    format!(
                        "Failed to set multiplied value {} to tape index {}",
                        result, result_offset
                    )
                })?;

                Ok(InstructionResult::Continue)
            }
            OpCode::Input => {
                print!("Input: ");
                stdout()
                    .flush()
                    .with_context(|| format!("Failed to flush stdout while reading input"))?;
                let mut input = String::new();
                stdin()
                    .read_line(&mut input)
                    .context("Failed to read input")?;

                let result_offset = self.get_argument_value(tape, 0)?;

                let value = input
                    .trim()
                    .parse()
                    .context("Failed to parse input string")?;

                debug!("[INP] {} -> [{}]", value, result_offset);

                tape.set(result_offset as usize, value).with_context(|| {
                    format!(
                        "Failed to set input value {} to tape index {}",
                        value, result_offset
                    )
                })?;

                Ok(InstructionResult::Continue)
            }
            OpCode::Output => {
                println!("[OUTPUT] {}", self.get_argument_value(tape, 0)?);

                Ok(InstructionResult::Continue)
            }
            OpCode::Terminate => Ok(InstructionResult::Terminate),
        }
    }
}

impl TryFrom<i64> for OpCode {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(OpCode::Add),
            2 => Ok(OpCode::Multiply),
            3 => Ok(OpCode::Input),
            4 => Ok(OpCode::Output),
            99 => Ok(OpCode::Terminate),
            _ => Err(format_err!("Unknown opcode {}", value)),
        }
    }
}

impl TryFrom<&str> for OpCode {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value_i: i64 = value
            .parse()
            .with_context(|| format!("Failed to parse opcode string into i64: \"{}\"", value))?;
        value_i.try_into()
    }
}

enum InstructionResult {
    Continue,
    Terminate,
}

fn main() -> Result<()> {
    env_logger::init();

    let mut tape = read_input("input.txt")?;

    let mut pc = 0;

    loop {
        let instruction = Instruction::new(&tape, pc)
            .with_context(|| format!("Failed to build instruction at offset {}", pc))?;
        match instruction
            .run(&mut tape)
            .with_context(|| format!("Failed to run instruction at offset {}", pc))?
        {
            InstructionResult::Continue => pc += instruction.opcode.argument_count() + 1,
            InstructionResult::Terminate => break,
        }
    }

    Ok(())
}
