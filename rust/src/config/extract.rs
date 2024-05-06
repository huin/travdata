use serde::Deserialize;

use crate::extraction::tableextract::groupers;

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
/// Configures the specifics of extracting the CSV from the PDF.
pub struct TableExtraction {
    pub transforms: Vec<TableTransform>,
}

#[derive(Deserialize, Debug)]
/// Supported table transformation operations.
pub enum TableTransform {
    ExpandColumnOnRegex(ExpandColumnOnRegex),
    FoldRows(FoldRows),
    JoinColumns(JoinColumns),
    PrependRow(PrependRow),
    Transpose(Transpose),
    WrapRowEveryN(WrapRowEveryN),
}

#[derive(Deserialize, Debug)]
/// Splits a column by the matches of a regex.
pub struct ExpandColumnOnRegex {
    pub column: usize,
    pub pattern: String,
    // When `pattern` matches (using `Pattern.fullmatch`), `on_match` produces
    // the resulting cells, using groups from the match to the pattern. Each
    // string is expanded using `Match.expand`, see
    // https://docs.python.org/3/library/re.html#match-objects.
    pub on_match: Vec<String>,
    // When `pattern` does not match, default produces cells as if matching on a
    // regex ".*" when `pattern` does not match. Similarly, each string is
    // expanded using `Match.expand` (using \g<0> makes sense here to extract the
    // entire original text into a cell).
    pub default: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Folds rows, according to the given sequence of groupings.
pub struct FoldRows {
    pub group_by: Vec<groupers::RowGrouper>,
}

#[derive(Deserialize, Debug)]
/// Joins a range of columns.
pub struct JoinColumns {
    #[serde(default = "Default::default")]
    pub from: Option<usize>,
    #[serde(default = "Default::default")]
    pub to: Option<usize>,
    #[serde(default = "Default::default")]
    pub delim: String,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Appends given literal row values to the start of a table.
pub struct PrependRow(pub Vec<String>);

#[derive(Deserialize, Debug)]
/// Transposes the table (rows become columns and vice versa).
pub struct Transpose {}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Wraps a row every N columns.
pub struct WrapRowEveryN {
    pub num_columns: usize,
}
