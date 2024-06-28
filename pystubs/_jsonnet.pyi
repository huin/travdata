from typing import Callable

class Undefined:
    pass

UNDEFINED = Undefined()

def evaluate_file(
    filename: str,
    max_stack: int | Undefined = UNDEFINED,
    gc_min_objects: int | Undefined = UNDEFINED,
    gc_growth_trigger: int | Undefined = UNDEFINED,
    ext_vars: dict[str, str] | Undefined = UNDEFINED,
    ext_codes: dict[str, str] | Undefined = UNDEFINED,
    tla_vars: dict[str, str] | Undefined = UNDEFINED,
    tla_codes: dict[str, str] | Undefined = UNDEFINED,
    max_trace: int | Undefined = UNDEFINED,
    import_callback: Callable[[str, str], tuple[str, bytes]] | Undefined = UNDEFINED,
    native_callbacks: dict[str, tuple[tuple[str, ...], Callable]] | Undefined = UNDEFINED,
) -> str: ...
def evaluate_snippet(
    filename: str,
    expr: str,
    max_stack: int | Undefined = UNDEFINED,
    gc_min_objects: int | Undefined = UNDEFINED,
    gc_growth_trigger: int | Undefined = UNDEFINED,
    ext_vars: dict[str, str] | Undefined = UNDEFINED,
    ext_codes: dict[str, str] | Undefined = UNDEFINED,
    tla_vars: dict[str, str] | Undefined = UNDEFINED,
    tla_codes: dict[str, str] | Undefined = UNDEFINED,
    max_trace: int | Undefined = UNDEFINED,
    import_callback: Callable[[str, str], tuple[str, bytes]] | Undefined = UNDEFINED,
    native_callbacks: dict[str, tuple[tuple[str, ...], Callable]] | Undefined = UNDEFINED,
) -> str: ...
