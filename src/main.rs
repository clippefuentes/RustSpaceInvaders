use std::{error::Error, time::Duration, sync::mpsc, thread};
use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
    cursor::{Hide, Show},
    event::{self, Event, KeyCode}
};
use invaders::{frame::{self, new_frame}, render};
use rusty_audio::Audio;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let mut audio = Audio::new();
    audio.add("explode", "sounds/original/explode.wav");
    audio.add("lose", "sounds/original/lose.wav");
    audio.add("move", "sounds/original/move.wav");
    audio.add("pew", "sounds/original/pew.wav");
    audio.add("startup", "sounds/original/startup.wav");
    audio.add("win", "sounds/original/win.wav");
    audio.play("startup");

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;

    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop seperate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });
    // Game loop
    'gameloop: loop {
        // Per frame init
        let curr_frame = new_frame();

        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }

        // Draw and render
       let _ = render_tx.send(curr_frame);
       thread::sleep(Duration:: from_millis(1));
    }
    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}
