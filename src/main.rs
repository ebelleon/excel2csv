use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};
use csv::WriterBuilder;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opt {
    /// Excel to be converted
    #[structopt(parse(from_os_str), short = "i", long = "input")]
    input: PathBuf,
    /// Destination for Converted CSV with file ending [default: input Name]
    #[structopt(parse(from_os_str), short = "o", long = "output")]
    output: Option<PathBuf>,
    /// Custom Delimiter
    #[structopt(short = "d", long = "delimiter", default_value = "|")]
    delimiter: char,
}

#[derive(Serialize)]
struct Row<'a> {
    #[serde(rename = "ExperienceProductID")]
    experience_product_id: &'a str,
    #[serde(rename = "OptionID")]
    option_id: &'a str,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let output = opt.output.map_or(default_output_path(&opt.input), Ok)?;
    convert_to_csv(opt.input, output, opt.delimiter as u8)
}

fn convert_to_csv(input: PathBuf, output: PathBuf, delimiter: u8) -> Result<(), Error> {
    let mut workbook: Xlsx<_> = open_workbook(input)?;
    let sheet = match workbook.worksheet_range("Tabelle1") {
        Some(workbook) => workbook?,
        None => return Err(Error::Msg("Cannot find 'Tabelle1'")),
    };

    let rows = RangeDeserializerBuilder::new().from_range(&sheet)?;

    let path = Path::new(&output);
    let mut wtr = WriterBuilder::new()
        .delimiter(delimiter)
        .from_path(path)
        .map_err(|e| Error::Io(e.into()))?;

    for result in rows {
        let (experience_product_id, option_id): (String, String) = result?;
        wtr.serialize(Row {
            experience_product_id: &experience_product_id.trim(),
            option_id: &option_id.trim(),
        })
        .map_err(|e| Error::Io(e.into()))?;
    }

    wtr.flush()?;

    remove_trailing_newline(output)
}

fn remove_trailing_newline(output: PathBuf) -> Result<(), Error> {
    let f = OpenOptions::new().write(true).open(&output)?;
    let metadata = fs::metadata(output)?;
    f.set_len(metadata.len() - 1)?;
    Ok(())
}

fn default_output_path(input: &PathBuf) -> Result<PathBuf, Error> {
    Ok(input
        .parent()
        .ok_or(Error::Msg("failed to get input's parent dir"))?
        .join(
            input
                .file_stem()
                .ok_or(Error::Msg("failed to get file stem"))?
                .to_str()
                .ok_or(Error::Msg("failed to convert os string"))?
                .to_owned()
                + ".csv",
        ))
}
