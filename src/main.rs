use anyhow::{Context, Error, Result};
use argh::FromArgs;
use fehler::throws;
use rand::{thread_rng, Rng};
use std::io::{BufRead, BufReader, Read};

/// Foobar
#[derive(FromArgs)]
struct PickerArgs {
    /// quux
    #[argh(option, short = 'n', default = "1")]
    num: usize,

    /// baz
    #[argh(switch, short = '0')]
    print0: bool,

    /// filename
    #[argh(positional)]
    filename: Option<String>,
}

#[derive(Debug)]
struct Picker<R>
where
    R: Read,
{
    lines_read: usize,
    num_to_choose: usize,
    chosen: Vec<String>,
    bufreader: Option<BufReader<R>>,
}

impl<R> Picker<R>
where
    R: Read,
{
    pub fn new(read: R, num: usize) -> Picker<R> {
        let bufreader = BufReader::new(read);
        let mut chosen = Vec::default();
        chosen.reserve(num);
        Picker {
            lines_read: 0,
            num_to_choose: num,
            chosen,
            bufreader: Some(bufreader),
        }
    }

    #[throws]
    pub fn pick(&mut self) {
        let mut rng = thread_rng();
        if let Some(bufreader) = self.bufreader.take() {
            for line in bufreader.lines() {
                if let Ok(l) = line {
                    self.process_line(l, &mut rng)?;
                } else {
                    line.with_context(|| "Error reading line.".to_string())?;
                }
            }
        }
    }

    #[throws]
    fn process_line(&mut self, line: String, rng: &mut impl Rng) {
        self.lines_read += 1;

        if self.chosen.len() < self.num_to_choose {
            self.chosen.push(line);
        } else {
            let val = rng.gen_range(0, self.lines_read);
            if val < self.num_to_choose {
                self.replace_line(line, rng);
            }
        }
    }

    fn replace_line(&mut self, line: String, rng: &mut impl Rng) {
        let index = rng.gen_range(0, self.chosen.len());
        self.chosen[index] = line;
    }

    pub fn spew(&self, print0: bool) {
        for chosen in &self.chosen {
            if print0 {
                print!("{}\0", chosen);
            } else {
                println!("{}", chosen);
            }
        }
    }
}

fn pick(filename: String, num: usize, print0: bool) -> Result<()> {
    let file = std::fs::File::open(&filename)
        .with_context(|| format!("Failed to open file at: {}", filename))?;
    let bufread = BufReader::new(file);
    let mut picker = Picker::new(bufread, num);
    picker.pick()?;
    picker.spew(print0);
    Ok(())
}

fn main() -> Result<()> {
    let args: PickerArgs = argh::from_env();
    if let Some(filename) = args.filename {
        pick(filename, args.num, args.print0)?;
    } else {
        let mut picker = Picker::new(std::io::stdin(), args.num);
        picker.pick()?;
        picker.spew(args.print0);
    }
    Ok(())
}
