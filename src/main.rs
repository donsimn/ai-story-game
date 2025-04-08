use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::{io, sync::Arc, sync::RwLock};
use tokio::runtime::Runtime;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Wrap},
    DefaultTerminal, Frame,
};

pub struct App {
    story: Arc<RwLock<String>>,
    health: Arc<RwLock<usize>>,
    exit: bool,

    ollama: Ollama,
    runtime: tokio::runtime::Runtime, // Initialize Tokio runtime
}

impl Default for App {
    fn default() -> Self {
        Self {
            story: Arc::new(RwLock::new(String::new())),
            health: Arc::new(RwLock::new(100)),
            exit: false,
            ollama: Ollama::new("http://localhost".to_string(), 11434), // Initialize Ollama
            runtime: Runtime::new().unwrap(), // Initialize Tokio runtime
        }
    }

}

impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('1') => {
                self.story.as_ref().write().unwrap().push_str("You chose option 1\n");
            },
            KeyCode::Char('2') => {
                self.story.as_ref().write().unwrap().push_str("You chose option 1\n");
            },
            KeyCode::Char('3') => {
                self.health -= 10;
            },
            KeyCode::Char('4') => {
                self.health += 10;
            },
            KeyCode::Char('5') => {
                self.story = "You chose option 5".to_string();
            },
            KeyCode::Char('6') => {
                self.generate_story("Write a short story about a dog named cat.");
            },
            _ => {}
        }
    }

        fn exit(&mut self) {
        self.exit = true;
    }

    fn generate_story(&mut self, prompt: &str) {
        let ollama = self.ollama.clone(); // Clone Ollama instance
        let prompt = prompt.to_string();
        let story_ref = self.story.clone();

        self.runtime.spawn(async move {
            let request = GenerationRequest::new("llama3".to_string(), prompt.to_string());
            match ollama.generate(request).await {
                Ok(response) => {
                    story_ref.push_str("\n");
                    story_ref.push_str(&response.response);
                }
                Err(e) => {
                    story_ref.push_str("\n[ERROR: Failed to generate response]");
                    eprintln!("Ollama error: {:?}", e);
                }
            }
        });
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal);
    ratatui::restore();
    
    //let _ollama = Ollama::new("http://localhost".to_string(), 11434);

    result
}


impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Story ".bold());
        let healthbar = Line::from(vec![
            format!(
                " {}{} ",
                "■".repeat(self.health / 2).red(),
                "□".repeat((100 - self.health) / 2)
            ).into(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(healthbar.centered())
            .border_set(border::THICK);

        // Ensure line breaks are handled
        let story_text = Text::from(self.story.clone()).yellow(); 

        Paragraph::new(story_text)
            .centered()
            .wrap(Wrap { trim: false }) // Enables text wrapping
            .block(block)
            .render(area, buf);
    }
}

