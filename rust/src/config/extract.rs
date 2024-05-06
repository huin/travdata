use serde::Deserialize;

#[derive(Deserialize, Debug)]
/// Supported table transformation operations.
pub enum TableTransform {
    ExpandColumnOnRegex(ExpandColumnOnRegex),
    PrependRow(PrependRow),
    Transpose(Transpose),
    FoldRows(FoldRows),
    JoinColumns(JoinColumns),
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
/// Appends given literal row values to the start of a table.
pub struct PrependRow(pub Vec<String>);

#[derive(Deserialize, Debug)]
/// Transposes the table (rows become columns and vice versa).
pub struct Transpose {}

#[derive(Deserialize, Debug)]
/// Suported configuring row grouping operations.
pub enum RowGrouper {
    AllRows(AllRows),
    StaticRowCounts(StaticRowCounts),
    EmptyColumn(EmptyColumn),
}

#[derive(Deserialize, Debug)]
/// Specifies to group all remaining rows.
pub struct AllRows {}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Specifies explicit input row counts for output grouped rows.
pub struct StaticRowCounts {
    pub row_counts: Vec<usize>,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Specifies to group rows by when a given column is empty.
pub struct EmptyColumn {
    pub column_index: usize,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Folds rows, according to the given sequence of groupings.
pub struct FoldRows {
    pub group_by: Vec<RowGrouper>,
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
/// Wraps a row every N columns.
pub struct WrapRowEveryN(pub usize);

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
/// Configures the specifics of extracting the CSV from the PDF.
pub struct TableExtraction {
    pub transforms: Vec<TableTransform>,
}
