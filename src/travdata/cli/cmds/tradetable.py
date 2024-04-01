# -*- coding: utf-8 -*-
"""
Produce a purchase DM table based on:

https://sirpoley.tumblr.com/post/643218580118323200/on-creating-a-frictionless-traveller-part-i
"""

import argparse
import codecs
import csv
import dataclasses
import enum
import io
import pathlib
import sys
import textwrap
import urllib.request
from typing import (
    Callable,
    Iterable,
    Iterator,
    NewType,
    Optional,
    Protocol,
    TypeAlias,
    TypeVar,
    cast,
)

from travdata import csvutil
from travdata.datatypes import yamlcodec
from travdata.datatypes.core import trade, worldcreation
from travdata.extraction import parseutil
from travdata.travellermap import apiurls, sectorparse, world

T = TypeVar("T")
# Maps from TradeGood.d66 to the lowest law level at which that good is illegal.
TradeGoodIllegality: TypeAlias = dict[str, int]


class UserError(Exception):
    """Raised for a problem detected with user input."""


class _IgnoreUnknown(enum.StrEnum):
    TRADE_CODES = "trade-codes"


# Trade overrides for a trade good on a world.
@dataclasses.dataclass
@yamlcodec.register_type
class WorldTradeOverrides:
    """Overrides specified on a specific world and trade good."""

    available: Optional[bool] = None
    purchase_dm: Optional[int] = None
    sale_dm: Optional[int] = None
    illegal: Optional[bool] = None


def _eval_override(v: T, override: Optional[T]) -> T:
    if override is not None:
        return override
    return v


_EMPTY_OVERRIDES = WorldTradeOverrides()


# Maps: [world location hex,trade good d66] -> overrides
WorldTradeOverridesMap: TypeAlias = dict[tuple[str, str], WorldTradeOverrides]


def _load_yaml(t: type[T], stream: pathlib.Path | io.TextIOBase) -> T:
    data = yamlcodec.DATATYPES_YAML.load(stream)
    if not isinstance(data, t):
        raise TypeError(type(data))
    return data


def _load_world_csv_data(path: str) -> Iterator[world.World]:
    """Parses a CSV file containing specific fields.

    These fields must be declared in the header row, and must include:

    - "Location" - the subsector hex location.
    - "Name" - the name of the world.
    - "UWP" - the UWP code.
    - "Trade Codes" - a colon (":") delimited list of two-letter trade codes.

    :param fp: File to read CSV data from.
    :yield: World data.
    """
    with csvutil.open_read(pathlib.Path(path)) as fp:
        r = csv.DictReader(fp)
        for row in r:
            yield world.World(
                name=row["Name"],
                location=world.WorldLocation(
                    subsector_hex=world.SubSectorLoc.parse(row["Location"]),
                ),
                uwp=worldcreation.UWP.parse(row["UWP"]),
                social=world.WorldSocial(
                    trade_codes=frozenset(
                        world.TradeCode(tc) for tc in row["Trade Codes"].split(":")
                    ),
                ),
            )


def _load_travellermap_tsv_file(path: str) -> list[world.World]:
    with open(path, "rt", encoding="utf-8") as fp:
        return list(sectorparse.t5_tsv(fp))


def _load_travellermap_tsv_url(url: str) -> list[world.World]:
    utf8 = codecs.lookup("utf-8")
    with urllib.request.urlopen(url) as response:
        decoder = cast(io.TextIOBase, utf8.streamreader(response))
        return list(sectorparse.t5_tsv(decoder))


def _load_travellermap_subsector(spec: str) -> list[world.World]:
    sector, slash, subsector_str = spec.partition("/")
    if not any([sector, slash, subsector_str]):
        raise UserError(
            "Invalid format for travellermap_subsector - expected"
            " sector/subsectorletter, e.g. spin/C"
        )
    url = apiurls.uwp_data(
        sector=apiurls.SectorId(sector),  # type: ignore[arg-type]
        subsector=apiurls.SubSectorCode[subsector_str],  # type: ignore[arg-type]
    )
    return _load_travellermap_tsv_url(url)


_WORLD_DATA_TYPES: dict[str, Callable[[str], Iterable[world.World]]] = {
    "csv": _load_world_csv_data,
    "travellermap_tsv_file": _load_travellermap_tsv_file,
    "travellermap_tsv_url": _load_travellermap_tsv_url,
    "travellermap_subsector": _load_travellermap_subsector,
}


def _pbool(s: str) -> bool:
    v = s.lower()
    if v == "true":
        return True
    if v == "false":
        return False
    raise ValueError(v)


def _load_world_trade_overrides(path: pathlib.Path) -> WorldTradeOverridesMap:
    result: WorldTradeOverridesMap = {}
    with csvutil.open_read(path) as fp:
        r = csv.DictReader(fp)
        for row in r:
            key = row["Location"], row["D66"]
            result[key] = WorldTradeOverrides(
                available=parseutil.map_opt_dict_key(_pbool, row, "Available"),
                purchase_dm=parseutil.map_opt_dict_key(int, row, "Purchase DM"),
                sale_dm=parseutil.map_opt_dict_key(int, row, "Sale DM"),
                illegal=parseutil.map_opt_dict_key(_pbool, row, "Illegal"),
            )
    return result


def _trade_dm(dms: dict[str, int], world_trades: set[str]) -> int:
    return max(
        (dms[wt] for wt in world_trades if wt in dms),
        default=0,
    )


def add_subparser(subparsers) -> None:
    """Adds a subcommand parser to ``subparsers``."""
    argparser: argparse.ArgumentParser = subparsers.add_parser(
        "tradetable",
        description=__doc__,
        formatter_class=argparse.RawTextHelpFormatter,
    )
    argparser.set_defaults(run=run)

    data_inputs_grp = argparser.add_argument_group("Data inputs")
    data_inputs_grp.add_argument(
        "data_dir",
        help="Path to the directory to read the Traveller YAML files from.",
        type=pathlib.Path,
        metavar="IN_DIR",
    )
    data_inputs_grp.add_argument(
        "--trade-good-illegality",
        help=textwrap.dedent(
            """
            Goods illegality data. Simple YAML mapping from trade good d66
            string to the numeric minimum law level at which it considered
            illegal.
            """
        ),
        type=argparse.FileType("rt"),
        metavar="trade-good-illegality.yaml",
    )
    data_inputs_grp.add_argument(
        "--world-trade-overrides",
        help=textwrap.dedent(
            """
            File containing trade overrides for the worlds. CSV file with
            columns: Location,D66,Available,Purchase DM,Sale DM,Illegal
            """
        ),
        type=pathlib.Path,
        metavar="world-trade-overrides.csv",
    )
    data_inputs_grp.add_argument(
        "world_data",
        help=textwrap.dedent(
            """
            World data within a single subsector. The meaning and interpretation
            of this parameter is determined by
            --world-data-source. An example of the type of URL required by
            travellermap_tsv_url is:
            https://travellermap.com/api/sec?sector=spin&subsector=A&type=TabDelimited
            """
        ),
        metavar="WORLD_DATA",
    )
    data_inputs_grp.add_argument(
        "--world-data-source",
        help="""Determines the meaning of the world_data argument.""",
        choices=_WORLD_DATA_TYPES.keys(),
        default="csv",
    )

    fmt_grp = argparser.add_argument_group(
        title="DM formatting",
        description=textwrap.dedent(
            """
            Extra formatting for DM cells, based on the goods status on a world.
            Each takes a single unamed argument `{}`, and is formatted according
            to Python str.format - see
            https://docs.python.org/3/library/stdtypes.html#str.format.
            """
        ),
    )
    fmt_grp.add_argument(
        "--format-common",
        help="When the goods are commonly available.",
        default="<b>{}</b>",
        metavar="FORMAT_STRING",
    )
    fmt_grp.add_argument(
        "--format-unavailable",
        help="When the goods is unavailable for purchase.",
        default="{}!",
        metavar="FORMAT_STRING",
    )
    fmt_grp.add_argument(
        "--format-legal",
        help="When the goods are legal.",
        default="{}",
        metavar="FORMAT_STRING",
    )
    fmt_grp.add_argument(
        "--format-illegal",
        help="When the goods are illegal.",
        default="<ul>{}</ul>",
        metavar="FORMAT_STRING",
    )

    argparser.add_argument(
        "--output-format",
        help="""Format of the output trading sheet.""",
        choices=_RESULT_WRITERS.keys(),
        metavar="FORMAT",
    )

    argparser.add_argument(
        "output_path",
        help="Path to the file to write.",
        type=pathlib.Path,
        metavar="FILE",
    )

    argparser.add_argument(
        "--ignore-unknowns",
        help="""Ignore unknown trade codes.""",
        choices=sorted(_IgnoreUnknown),
        action="append",
    )

    inc_grp = argparser.add_argument_group("Extra information to include")
    inc_grp.add_argument(
        "--include-headers",
        help="Include the table headers.",
        type=bool,
        action=argparse.BooleanOptionalAction,
        default=True,
    )
    inc_grp.add_argument(
        "--include-key",
        help="Include a key to the DM formatting.",
        type=bool,
        action=argparse.BooleanOptionalAction,
        default=True,
    )
    inc_grp.add_argument(
        "--include-explanation",
        help="Include explanation for how to use the table.",
        type=bool,
        action=argparse.BooleanOptionalAction,
        default=True,
    )


def _not_none(v: Optional[T], desc: str) -> T:
    if v is None:
        raise ValueError(f"{desc} was missing a value")
    return v


def _must_world_trade_codes(w: world.World) -> frozenset[world.TradeCode]:
    return _not_none(_not_none(w.social, "world social data").trade_codes, "world trade codes")


def _must_world_subsector_loc(w: world.World) -> str:
    return str(
        _not_none(
            _not_none(w.location, "world location data").subsector_hex,
            "world subsector location",
        )
    )


def _must_world_law_level(w: world.World) -> int:
    return _not_none(w.uwp, "world UWP data").law_level


# _WorldId is an arbitrary ID assigned to a world internally.
_WorldId = NewType("_WorldId", int)


@dataclasses.dataclass
class _WorldView:
    id: _WorldId
    # Full trade code name.
    trade_classifications: set[str]
    subsector_loc: str
    law_level: int


def _make_world_views(
    worlds: list[world.World],
    tcodes: list[worldcreation.TradeCode],
    ignore_unknowns: list[_IgnoreUnknown],
) -> list[_WorldView]:
    """Constructs a view onto the aspects of the worlds that we need."""
    world_views: list[_WorldView] = []
    tcodes_by_code = {tc.code: tc for tc in tcodes}
    for i, w in enumerate(worlds):
        trade_classifications: set[str] = set()
        for tc in _must_world_trade_codes(w):
            try:
                trade_code = tcodes_by_code[tc]
            except KeyError as e:
                if _IgnoreUnknown.TRADE_CODES not in ignore_unknowns:
                    raise UserError(
                        f"unknown trade code {e}, use --ignore-unknowns=trade-codes to ignore"
                    ) from e
            else:
                trade_classifications.add(trade_code.classification)
        world_views.append(
            _WorldView(
                id=_WorldId(i),
                trade_classifications=trade_classifications,
                subsector_loc=_must_world_subsector_loc(w),
                law_level=_must_world_law_level(w),
            )
        )
    return world_views


@dataclasses.dataclass
class _ResultDMData:
    world_view: _WorldView
    dm: int
    common: bool
    available: bool
    illegal: bool


# Sentences explaining the meaning of ``_ResultDMData.dm``.
_DM_EXPLANATION_SENTENCES = [
    "Number is added when buying goods, and subtracted when selling goods.",
    "High numbers indicate excess of supply, low numbers indicate demand.",
]


@dataclasses.dataclass
class _ResultTradeGoodDMs:
    tgood: trade.TradeGood
    dms: Optional[dict[_WorldId, _ResultDMData]]


def _calculate_trades(
    tgoods: list[trade.TradeGood],
    tgood_illegality: TradeGoodIllegality,
    world_views: list[_WorldView],
    wt_overrides: WorldTradeOverridesMap,
) -> Iterator[_ResultTradeGoodDMs]:
    """Core calculation of trade DMs and properties.

    :param tgoods: Trade goods.
    :param tgood_illegality: General illegality of goods (minimum law level of
    good being illegal).
    :param world_views: Worlds to produce data for.
    :param wt_overrides: Per-world calculated value overrides of trade good
    properties (such as illegality, purchase/sale DMs).
    :yield: Resulting trade data.
    """
    for tgood in tgoods:
        if not tgood.properties:
            yield _ResultTradeGoodDMs(tgood=tgood, dms=None)
            continue

        tprops = tgood.properties
        illegality = tgood_illegality.get(tgood.d66, None)

        dms: dict[_WorldId, _ResultDMData] = {}
        for world_view in world_views:
            overrides = wt_overrides.get((world_view.subsector_loc, tgood.d66), _EMPTY_OVERRIDES)
            dms[world_view.id] = _calculate_world_trade(tprops, overrides, world_view, illegality)

        yield _ResultTradeGoodDMs(tgood=tgood, dms=dms)


def _calculate_world_trade(
    tprops: trade.TradeGoodProperties,
    overrides: WorldTradeOverrides,
    world_view: _WorldView,
    illegality: Optional[int],
) -> _ResultDMData:
    is_common = "All" in tprops.availability

    is_available = is_common or not tprops.availability.isdisjoint(world_view.trade_classifications)
    is_available = _eval_override(is_available, overrides.available)

    purchase_dm = _trade_dm(tprops.purchase_dm, world_view.trade_classifications)
    purchase_dm = _eval_override(purchase_dm, overrides.purchase_dm)

    illegal = illegality is not None and world_view.law_level >= illegality
    illegal = _eval_override(illegal, overrides.illegal)
    if illegal and illegality is not None:
        sale_dm = world_view.law_level - illegality
    else:
        sale_dm = _trade_dm(tprops.sale_dm, world_view.trade_classifications)
    sale_dm = _eval_override(sale_dm, overrides.sale_dm)

    dm = purchase_dm - sale_dm

    return _ResultDMData(
        world_view=world_view,
        common=is_common,
        illegal=illegal,
        dm=dm,
        available=is_available,
    )


@dataclasses.dataclass
class _OutputOpts:
    include_headers: bool
    include_key: bool
    include_explanation: bool
    formats: "_Formats"


@dataclasses.dataclass
class _Formats:
    common: str
    unavailable: str
    illegal: str
    legal: str

    def fmt_dm(self, result_dm: _ResultDMData) -> str:
        """Formats the given DM, based on its properties.

        :param result_dm: DM and its properties.
        :return: Formatted DM.
        """
        fmts = []
        if result_dm.common:
            fmts.append(self.common)
        if not result_dm.available:
            fmts.append(self.unavailable)
        if result_dm.illegal:
            fmts.append(self.illegal)
        else:
            fmts.append(self.legal)
        dm_str = f"{result_dm.dm:+}"

        for fmt in fmts:
            dm_str = fmt.format(dm_str)
        return dm_str

    def key(self, example_dm: str) -> list[tuple[str, str]]:
        """Formats and returns the given DM with explanation as a legend.

        :param example_dm: Example DM text to format in each case.
        :return: Tuples, each of which containing the formatted DM, and the
        corresponding description of its meaning.
        """
        entries = [
            (self.common, "Commonly available goods."),
            (self.unavailable, "Good unavailable for purchase."),
            (self.legal, "Legal by planetary law."),
            (self.illegal, "Illegal by planetary law."),
        ]
        return [(fmt.format(example_dm), explanation) for fmt, explanation in entries]


class _ResultWriter(Protocol):  # pylint: disable=too-few-public-methods
    def __call__(
        self,
        *,
        output_path: pathlib.Path,
        world_views: list[_WorldView],
        tgood_results: list[_ResultTradeGoodDMs],
        opts: _OutputOpts,
    ) -> None: ...


def _write_results_asciidoc(
    *,
    output_path: pathlib.Path,
    world_views: list[_WorldView],
    tgood_results: list[_ResultTradeGoodDMs],
    opts: _OutputOpts,
) -> None:
    with open(output_path, "wt", encoding="utf-8") as fp:

        def writeln(s: str = "") -> None:
            print(s, file=fp)

        def writecell(s: str, duplication: int = 1, operators: str = "") -> None:
            if duplication == 1:
                print(f"{operators}|{s}", file=fp)
            else:
                print(f"{duplication}*{operators}|{s}", file=fp)

        writeln("= Trading DM Table")
        writeln()

        col_specs = ["1", "4", "2", "3", f"{len(world_views)}*1"]
        writeln(f"[cols=\"{','.join(col_specs)}\"]")
        writeln("|===")  # Start of table content.
        writeln(
            "|"
            + " |".join(
                [
                    "D66",
                    "Goods",
                    "Tons",
                    "Base Price (cr)",
                ]
                + [world_view.subsector_loc for world_view in world_views]
            )
        )

        for tgood_result in tgood_results:
            writeln()  # Start of new row.
            writecell(tgood_result.tgood.d66)
            writecell(tgood_result.tgood.name)

            if tprops := tgood_result.tgood.properties:
                writecell(tprops.tons)
                writecell(str(tprops.base_price))
            else:
                writecell("", duplication=2)

            if not tgood_result.dms:
                writecell("", duplication=len(world_views))
                continue

            for world_view in world_views:
                world_dm = tgood_result.dms.get(world_view.id)
                if not world_dm:
                    writecell("")
                    continue
                writecell(opts.formats.fmt_dm(world_dm), operators="m")

        writeln("|===")  # End of table content.

        if opts.include_key:
            writeln()
            writeln("== Key")
            for key_item, explanation in opts.formats.key("+2"):
                writeln(f"{explanation}:: `{key_item}`")

        if opts.include_explanation:
            writeln()
            writeln("== How to use")
            for s in _DM_EXPLANATION_SENTENCES:
                writeln(f"- {s}")


def _write_results_csv(
    *,
    output_path: pathlib.Path,
    world_views: list[_WorldView],
    tgood_results: list[_ResultTradeGoodDMs],
    opts: _OutputOpts,
) -> None:
    with csvutil.open_write(output_path) as fp:
        csv_writer = csv.writer(fp)
        if opts.include_headers:
            csv_writer.writerow(
                ["D66", "Goods", "Tons", "Base Price (cr)"] + [w.subsector_loc for w in world_views]
            )

        for tgood_result in tgood_results:
            tgood = tgood_result.tgood
            row = [tgood.d66, tgood.name]

            if tprops := tgood_result.tgood.properties:
                row.append(tprops.tons)
                row.append(str(tprops.base_price))
            else:
                row.extend(["", ""])

            if not tgood_result.dms:
                csv_writer.writerow(row)
                continue

            for world_view in world_views:
                world_dm = tgood_result.dms.get(world_view.id)
                if not world_dm:
                    row.append("")
                    continue
                row.append(opts.formats.fmt_dm(world_dm))

            csv_writer.writerow(row)

        if opts.include_key:
            csv_writer.writerow([])
            csv_writer.writerow(["Key:"])
            for key_item, explanation in opts.formats.key("+2"):
                csv_writer.writerow([key_item, explanation])

        if opts.include_explanation:
            csv_writer.writerow([])
            for s in _DM_EXPLANATION_SENTENCES:
                csv_writer.writerow([s])


_RESULT_WRITERS: dict[str, _ResultWriter] = {
    "asciidoc": _write_results_asciidoc,
    "csv": _write_results_csv,
}


def process(args: argparse.Namespace) -> None:
    """Runs the program, given the parsed command line arguments.

    :param args: Command line arguments.
    """
    tcodes = cast(
        list[worldcreation.TradeCode],
        _load_yaml(list, args.data_dir / worldcreation.GROUP / "trade-codes.yaml"),
    )
    tgoods = cast(
        list[trade.TradeGood], _load_yaml(list, args.data_dir / trade.GROUP / "trade-goods.yaml")
    )
    tgood_illegality: TradeGoodIllegality
    if args.trade_good_illegality:
        tgood_illegality = cast(TradeGoodIllegality, _load_yaml(dict, args.trade_good_illegality))
    else:
        tgood_illegality = {}
    if args.world_trade_overrides:
        wt_overrides = _load_world_trade_overrides(args.world_trade_overrides)
    else:
        wt_overrides = {}

    world_reader = _WORLD_DATA_TYPES[args.world_data_source]
    worlds = list(world_reader(args.world_data))

    world_views = _make_world_views(worlds, tcodes, args.ignore_unknowns)

    tgood_results = list(
        _calculate_trades(
            tgoods=tgoods,
            tgood_illegality=tgood_illegality,
            world_views=world_views,
            wt_overrides=wt_overrides,
        )
    )

    result_writer = _RESULT_WRITERS[args.output_format]
    result_writer(
        output_path=args.output_path,
        tgood_results=tgood_results,
        world_views=world_views,
        opts=_OutputOpts(
            include_headers=args.include_headers,
            formats=_Formats(
                common=args.format_common,
                unavailable=args.format_unavailable,
                legal=args.format_legal,
                illegal=args.format_illegal,
            ),
            include_explanation=args.include_explanation,
            include_key=args.include_key,
        ),
    )


def run(args: argparse.Namespace) -> None:
    """Entrypoint for the program."""
    try:
        process(args)
    except UserError as e:
        print(f"Error: {e}", file=sys.stderr)
