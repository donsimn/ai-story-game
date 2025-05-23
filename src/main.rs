use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Rgb;
use ollama_rs::generation::parameters::JsonStructure;
use ollama_rs::{generation::parameters::FormatType, Ollama};
use ollama_rs::generation::completion::request::GenerationRequest;
use serde::Deserialize;
use serde_json::json;
use std::{io, ops::{AddAssign, SubAssign}, sync::{Arc, RwLock}};
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


#[derive(ollama_rs::generation::parameters::JsonSchema, Deserialize, Debug)]
struct OutputSchema {
    story: String,
    health: usize,
    options: Vec<String>,
}

pub struct App {
    story: String,
    health: usize,
    current_options: Vec<String>,
    exit: bool,

    ollama: Ollama,
    runtime: tokio::runtime::Runtime, // Initialize Tokio runtime
}

impl Default for App {
    fn default() -> Self {
        Self {
            story: String::new(),
            health: 100,
            current_options: vec![],
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
            KeyCode::Char('g') => {
            
                let prompt = format!("You are an interactive story generator for an AI game. ¸\nTheme: {}\nGenerate a story that follows this structure:\n\n1. Story & Setting:\n- Create a vivid, immersive narrative based on the theme.\n- Introduce the main character and the world they’re in.\n- The story at the beginning should be a minimum of 3 and a maximum of 6 sentences long.\n\n2. Health Bar Mechanic:\n- The main character starts with a full health bar (e.g., 100).\n- Throughout the story, include events that either damage or heal them.\n- Clearly indicate all health changes.\n\n3. Decision Point:\n- End the story with a moment where the player must choose between at least two actions.\n- List each option clearly, and briefly explain the outcome and health impact.\n- If the health bar drops to 0, the character dies and the game ends.\n\nOutput format:\n- Story: The full narrative leading up to the decision.\n- Health: The current health value and how it changed.\n- Choices: A structured list of available decisions with consequences", "Short, janos alone in the desert");

                self.generate_story(&prompt);
            },
            KeyCode::Char(ch) => {
                if ch.is_ascii_digit() {
                    let index = ch.to_digit(10).unwrap() as usize;
                    if index > 0 && index <= self.current_options.len() {
                        
                        let option = &self.current_options[index - 1];
                        self.story.push_str(&format!("You chose option {}: {}\n", index, option));

                        
                        let prompt = format!("You are an interactive story generator for an AI game that continues from previous decisions. Generate the next part of the story based on the player’s last choice and previous events. Follow this structure:\n\n1. Continue the Story:\n- Use the previous story, healthpoints and the player’s selected option.\n- Write a new story segment (3 to 6 sentences long) that logically follows from the decision.\n- Include the direct consequences of the player’s decision in this segment (e.g., gained an item, took damage, found something hidden).\n\n2. Health Bar Mechanic:\n- The character’s health should only be influenced by the player’s choices.\n- Reflect any health gain or loss that resulted from the previous decision.\n- Update and display the health bar before presenting the next decision.\n\n3. Decision Point:\n- Present at least two new choices for the player.\n- List each option clearly and explain the expected outcome and its impact on the character’s health.\n- Do not continue the story after presenting the choices — stop and wait for the player to select an option.\n- If the health bar reaches 0, the character dies and the game ends.\n\nOutput format:\n- Story: A 3–6 sentence narrative that reflects the last decision and sets up the next choice.\n- Health: The current health value and how it changed (e.g., Health: 75 (-10)).\n- Choices: A structured list of new available decisions with consequences and health impact. This what happened previously with the players choises: \n\n {}",self.story);

                        self.generate_story(&prompt);

                        self.current_options.clear(); // Clear options after selection
                    
                    } else {
                        self.story.push_str("Invalid option selected.\n");
                    }
                }
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
        let story_ref = Arc::new(RwLock::new(self.story.clone()));
        let health_ref = Arc::new(RwLock::new(self.health.clone()));
        let options_ref : Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(vec![]));

        let finnished = Arc::new(RwLock::new(false));

        let answer = Arc::new(RwLock::new(String::new()));


    
        let story_clone = story_ref.clone();
        let health_clone = health_ref.clone();
        let options_clone = options_ref.clone();

        let finnished_clone = finnished.clone();

        let answer_clone = answer.clone();

        self.runtime.spawn(async move {


            let format = FormatType::StructuredJson(JsonStructure::new::<OutputSchema>());

            let request = GenerationRequest::new("llama3.2".to_string(), prompt.to_string()).format(format);
            match ollama.generate(request).await {
                Ok(response) => {
                    story_clone.as_ref().write().unwrap().push_str("\n");
                    story_clone.as_ref().write().unwrap().push_str(&serde_json::from_str::<OutputSchema>(&response.response).unwrap().story);
                    *health_clone.write().unwrap() = serde_json::from_str::<OutputSchema>(&response.response).unwrap().health;
                    *options_clone.write().unwrap() = serde_json::from_str::<OutputSchema>(&response.response).unwrap().options;

                    answer_clone.as_ref().write().unwrap().push_str(&response.response);

                    *finnished_clone.write().unwrap() = true;
                }
                Err(e) => {
                    story_clone.as_ref().write().unwrap().push_str("\n[ERROR: Failed to generate response]");
                    eprintln!("Ollama error: {:?}", e);
                }
            }
        });

        loop {
            if *finnished.read().unwrap() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }  

        self.story = story_ref.read().unwrap().clone();
        self.health = *health_ref.read().unwrap();
        let options = options_ref.read().unwrap().clone();

        if !options.is_empty() {

            self.current_options = options.clone();

            self.story.push_str("\n\nOptions:\n");
            for (i, option) in options.iter().enumerate() {
                let color = Rgb(0, 255 / (options.len() as u8 + 1) * (i as u8 +2), 0);
                self.story.push_str(&format!("{}: {}\n", i + 1, option).color(color).to_string());
            }
        } else {
            self.story.push_str("\n\nNo options available.\n");
        }
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal);
    ratatui::restore();
    
    result
}


impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Story ".bold());
        let healthbar = Line::from(vec![
            format!(
                " {}{} ",
                "■".repeat(self.health.clone() / 2).red(),
                "□".repeat((100 - self.health.clone()) / 2)
            ).into(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(healthbar.centered())
            .border_set(border::THICK);

        // Ensure line breaks are handled
        let story_text = Text::from(self.story.clone()); 

        Paragraph::new(story_text)
            .centered()
            .wrap(Wrap { trim: false }) // Enables text wrapping
            .block(block)
            .render(area, buf);
    }
}

