/**
 * Created by sofia on 2023-03-08.
 * Source: Udemy Ultimate Rust Crash Course
 */

use std::error::Error;
use std::{io, thread};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use crossterm::cursor::{Hide, Show};
use crossterm::{event, ExecutableCommand};
use crossterm::event::{Event, KeyCode};
use crossterm::terminal;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use rusty_audio::Audio;
use invaders::{frame, render};
use invaders::frame::{Drawable, new_frame};
use invaders::invaders::Invaders;
use invaders::player::Player;

fn main() -> Result<(), Box<dyn Error>> {
    /****************
     * Audio
     ****************/
    let mut audio = Audio::new();

    audio.add("explode", "audio/explode.wav");
    audio.add("lose", "audio/lose.wav");
    audio.add("move", "audio/move.wav");
    audio.add("pew", "audio/pew.wav");
    audio.add("startup", "audio/startup.wav");
    audio.add("win", "audio/win.wav");

    audio.play("startup");

    /****************
     * Terminal
     ****************/
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    /****************
     * Render Loop
     ****************/
    let (render_tx, render_rx) = mpsc::channel();

    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();

        render::render(&mut stdout, &last_frame, &last_frame, true);

        loop {
            let mut curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };

            render::render(&mut stdout, &mut last_frame, &mut curr_frame, false);

            last_frame = curr_frame;
        }
    });

    /****************
     * Game Loop
     ****************/
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();

    'game_loop: loop {
        // Per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    },
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'game_loop;
                    },
                    _ => {}
                }
            }
        }

        // Updates
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        if player.detect_hits(&mut invaders) {
            audio.play("explode");
        }

        // Draw & render
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        // Win or lose?
        if invaders.all_killed() {
            audio.play("win");
            break 'game_loop;
        }
        if invaders.reached_bottom() {
            audio.play("lose");
            break 'game_loop;
        }
    }

    /****************
     * Cleanup
     ****************/
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())

}
