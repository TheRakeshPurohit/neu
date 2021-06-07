use crate::state::{Mode, State};
use std::io::{stdout, BufWriter, Stdout, Write};
use termion::raw::{IntoRawMode, RawTerminal};
use xtra::prelude::*;

pub(crate) struct Renderer {
    stdout: BufWriter<RawTerminal<Stdout>>,
}

impl Renderer {
    pub(crate) fn new() -> Self {
        let mut stdout = BufWriter::new(stdout().into_raw_mode().unwrap());
        write!(stdout, "{}", termion::screen::ToAlternateScreen).unwrap();
        stdout.flush().unwrap();
        Self { stdout }
    }
}

impl Actor for Renderer {}

pub(crate) struct Render(pub(crate) State);

impl Message for Render {
    type Result = ();
}

#[async_trait::async_trait]
impl Handler<Render> for Renderer {
    async fn handle(&mut self, msg: Render, _ctx: &mut Context<Self>) {
        let state = msg.0;

        write!(
            self.stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::All
        )
        .unwrap();
        let textarea_row = state.size.1 - 2;
        let max_line_digit = format!("{}", state.buffer.lines().count()).chars().count();
        let textarea_col = state.size.0 - max_line_digit as u16 - 1;
        for (i, line) in state.buffer.lines().skip(state.row_offset).take(textarea_row as usize).enumerate() {
            write!(
                self.stdout,
                "{:max_line_digit$} {}\r\n",
                state.row_offset + i + 1,
                line.chars().take(textarea_col as usize).collect::<String>(),
                max_line_digit = max_line_digit
            )
            .unwrap();
        }
        write!(
            self.stdout,
            "{}",
            termion::cursor::Goto(0, state.size.1 - 1)
        )
        .unwrap();
        match &state.mode {
            Mode::Normal(cmd) => {
                if cmd.is_empty() {
                    write!(self.stdout, "{}NORMAL", termion::cursor::SteadyBlock).unwrap();
                } else {
                    write!(self.stdout, "{}NORMAL", termion::cursor::SteadyUnderline).unwrap();
                }
            }
            Mode::Insert => {
                write!(self.stdout, "{}INSERT", termion::cursor::SteadyBar).unwrap();
            }
            Mode::CmdLine(cmd) => {
                write!(
                    self.stdout,
                    "{}COMMAND{}:{}",
                    termion::cursor::SteadyBlock,
                    termion::cursor::Goto(0, state.size.1),
                    cmd
                )
                .unwrap();
            }
        };
        let col = state.cursor.col;
        let row = state.cursor.row;
        write!(
            self.stdout,
            "{}",
            termion::cursor::Goto((max_line_digit + col + 2) as u16, row as u16 + 1)
        )
        .unwrap();
        self.stdout.flush().unwrap();
    }
}

pub(crate) struct Finish;

impl Message for Finish {
    type Result = ();
}

#[async_trait::async_trait]
impl Handler<Finish> for Renderer {
    async fn handle(&mut self, _msg: Finish, _ctx: &mut Context<Self>) {
        write!(
            self.stdout,
            "{}{}",
            termion::clear::All,
            termion::screen::ToMainScreen
        )
        .unwrap();
        self.stdout.flush().unwrap();
    }
}
