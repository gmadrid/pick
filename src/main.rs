use anyhow::{Context, Error, Result};
use argh::FromArgs;
use fehler::throws;
use rand::{thread_rng, Rng};
use std::io::{BufRead, BufReader, Read};

/// Implementation of Knuth's algorithm to pick n items from M,
/// each with identical likelihood of inclusion.
/// Uses O(n) space.
#[derive(FromArgs)]
struct PickerArgs {
    /// the number of items to pick
    #[argh(option, short = 'n', default = "1")]
    num: usize,

    /// print with zeros, '\0', between lines instead of '\n'.
    /// This is useful when piping lines that contain spaces.
    #[argh(switch, short = '0')]
    print0: bool,

    /// the file to read. If not provided, uses standard input.
    #[argh(positional)]
    filename: Option<String>,
}

#[derive(Debug)]
struct Picker {
    // The number of lines read so far.
    // Used in the calculation 1/lines_read which is the probability of picking any new line.
    lines_read: usize,

    // The total number of items we wish to pick.
    num_to_choose: usize,

    // The lines from the input which have already been chosen.
    chosen: Vec<String>,
    // TODO: get rid of this!
    //bufreader: Option<BufReader<R>>,
}

impl Picker {
    pub fn new(num_to_choose: usize) -> Picker {
        let mut chosen = Vec::default();
        chosen.reserve(num_to_choose);
        Picker {
            lines_read: 0,
            num_to_choose,
            chosen,
        }
    }

    #[throws]
    pub fn pick(&mut self, reader: impl Read) {
        let mut rng = thread_rng();
        let bufreader = BufReader::new(reader);
        for line in bufreader.lines() {
            if let Ok(l) = line {
                self.process_line(l, &mut rng)?;
            } else {
                line.with_context(|| "Error reading line.".to_string())?;
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

#[throws]
fn pick_and_spew(r: impl Read, num: usize, print0: bool) {
    let mut picker = Picker::new(num);
    picker.pick(r)?;
    picker.spew(print0);
}

fn main() -> Result<()> {
    let args: PickerArgs = argh::from_env();
    if let Some(filename) = args.filename {
        let file = std::fs::File::open(&filename)
            .with_context(|| format!("Failed to open file at: {}", filename))?;
        pick_and_spew(file, args.num, args.print0)?;
    } else {
        pick_and_spew(std::io::stdin(), args.num, args.print0)?;
    }
    Ok(())
}
