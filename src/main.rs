mod color;

use std::{io::Write, time::Duration, vec};
use crossterm::{cursor, queue, style, terminal};
use anyhow::{Context, Result};
use color::{Color, HslColor};
use rand::{thread_rng, Rng};

//const DICT: &'static str = "ﾊﾐﾋｰｳｼﾅﾓﾆｻﾜﾂｵﾘｱﾎﾃﾏｹﾒｴｶｷﾑﾕﾗｾﾈｽﾀﾇﾍｦｲｸｺｿﾁﾄﾉﾌﾔﾖﾙﾚﾛﾝ012345789Z:.\"=*+-<>¦╌ç";
const DICT: &'static str = "abcdefghijklmnopqrstuvwxyz123456789:.-!<>?/\\+=\"*";
const DICT_LEN: usize = DICT.len();

#[derive(Clone)]
struct Glyph {
    character: char,
    color: Color,
}

impl Default for Glyph {
    fn default() -> Self {
        Self {
            character: ' ',
            color: Color::from_rgb(0, 0, 0),
        }
    }
}

impl Glyph {
    fn new(character: char, color: Color) -> Self {
        Self {
            character,
            color,
        }
    }

    fn render<W: Write>(&self, out: &mut W) -> Result<()> {
        queue!(
            out,
            style::SetForegroundColor(style::Color::Rgb {
                r: self.color.r,
                g: self.color.g,
                b: self.color.b
            })
        )?;
        queue!(out, style::Print(self.character.to_string())).context("write glyph to output")?;

        Ok(())
    }

    fn fade_color(&mut self) {
        let hsl = self.color.as_hsl();
        let new_color = HslColor::new(hsl.h, hsl.s * 0.90, hsl.l * 0.90);
        if new_color.s < 10.0 || new_color.l < 10.0 {
            self.color = HslColor::new(hsl.h, 10.0, 10.0).into();
        } else {
            self.color = new_color.into();
        }
    }
}

#[derive(Clone)]
struct Column {
    height: u16,
    basic_color: Color,
    glyphs: Vec<Glyph>,
    active_index: usize,//why usize
}

impl Column {
    fn new(height: u16, basic_color: Color, ) -> Self {
        Self {
            height,
            basic_color,
            glyphs: vec![Glyph::default(); height as usize],
            active_index: 0,
        }
    }

    fn render<W: Write>(&self, out: &mut W, y: u16) -> Result<()> {
        self.glyphs[y as usize].render(out)?;

        Ok(())
    }

    fn step<R: Rng>(&mut self, rand: &mut R) {
        for glyph in self.glyphs.iter_mut() {
            glyph.fade_color();
        }

        if (self.active_index == 0) && (rand.gen_range(0..100) < 90) {
            return;
        }

        let id = rand.gen_range(0..DICT_LEN);
        self.glyphs[self.active_index] = Glyph::new(DICT.chars().nth(id).unwrap(), self.basic_color);
        //self.glyphs[self.active_index] = Glyph::new('a', self.basic_color);
        self.active_index += 1;
        
        if self.active_index >= self.height as usize{
            self.active_index = 0;
        }
    }
}

struct MatrixWaterfall {
    height: u16,
    width: u16,
    basic_color: Color,
    columns: Vec<Column>,
}

impl MatrixWaterfall {
    fn new(height: u16, width: u16, basic_color: Color) -> Self {
        Self {
            height,
            width,
            basic_color,
            columns: vec![Column::new(height, basic_color); width as usize],
        }
    }

    fn render<W: Write>(&self, out: &mut W) -> Result<()> {
        queue!(out, cursor::Hide)?;
        queue!(out, cursor::MoveTo(0, 0))?;
        queue!(out, style::SetBackgroundColor(style::Color::Black))?;

        for y in 0..self.height {
            for col in self.columns.iter() {//equals to "&self.columns?"
                col.render(out, y)?;
            }
        }

        queue!(out, style::ResetColor)?;
        queue!(out, cursor::Show)?;
        out.flush().context("flush")?;
        Ok(())
    }

    fn step(&mut self) {
        let mut rng = thread_rng();
        for col in self.columns.iter_mut() {//equals to "&mut self.columns"?
            col.step(&mut rng);
        }
    }
}

fn main() -> Result<()> {
    let (width, height) = terminal::size().context("Take the Terminal Size")?;
    let mut waterfall = MatrixWaterfall::new(height, width, Color::from_rgb(0, 255, 43));

    let mut stdout = std::io::stdout();
    loop {
        waterfall.render(&mut stdout)?;
        waterfall.step();
        //sleeps 150ms after every flushing
        std::thread::sleep(Duration::from_millis(100));
    }
}