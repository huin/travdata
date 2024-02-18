# -*- coding: utf-8 -*-
import io
import json
from typing import (Any, AnyStr, Callable, ClassVar, Protocol, TypeAlias,
                    TypeVar, runtime_checkable)

_TYPE_FIELD = "__type__"


Object: TypeAlias = dict[str, Any]


@runtime_checkable
class Encodable(Protocol):

    @classmethod
    def json_type(cls) -> str: ...

    def to_json(self) -> Object: ...


class Decodable(Protocol):

    @classmethod
    def json_type(cls) -> str: ...

    @classmethod
    def from_json(cls, o: Object) -> "Decodable": ...


_JSONEncFn: TypeAlias = Callable[[Any], Object]
_JSONDecFn: TypeAlias = Callable[[Object], Any]


JD = TypeVar("JD", bound=type[Decodable])


class _Encoder(json.JSONEncoder):
    _encode_adapt: ClassVar[dict[type, _JSONEncFn]]

    def default(self, o: Any):
        if isinstance(o, Encodable):
            return o.to_json()
        elif enc := self._encode_adapt.get(type(o)):
            return enc(o)
        return super().default(o)


class Codec:
    _decode_adapt: dict[str, _JSONDecFn]
    _encode_adapt: dict[type, _JSONEncFn]
    _encoder: type[_Encoder]

    def __init__(self) -> None:
        self._decode_adapt = {}
        self._encode_adapt = {}

        # Because the JSON library requires a class (not an instance) to inject
        # the encoder behaviour, we must create an inner subclass here. Eww.
        class Encoder(_Encoder):
            _encode_adapt = self._encode_adapt

        self._encoder = Encoder

    def self_register_builtins(self) -> None:
        self._decode_adapt["set"] = lambda v: set(v["v"])
        self._encode_adapt[set] = lambda v: {_TYPE_FIELD: "set", "v": list(v)}

    def register_json_decodable(self, cls: JD) -> JD:
        self._decode_adapt[cls.json_type()] = cls.from_json
        return cls

    def _json_object_hook(self, o: Object) -> Any:
        if tname := o.pop(_TYPE_FIELD):
            return self._decode_adapt[tname](o)
        else:
            return o

    def dumps(self, obj: Any) -> str:
        return json.dumps(obj, cls=self._encoder)

    def dump(self, obj: Any, fp: io.TextIOBase) -> None:
        json.dump(obj=obj, fp=fp, cls=self._encoder)

    def loads(self, s: AnyStr) -> Any:
        return json.loads(s, object_hook=self._json_object_hook)

    def load(self, fp: io.TextIOBase) -> Any:
        return json.load(fp=fp, object_hook=self._json_object_hook)


DEFAULT_CODEC = Codec()
DEFAULT_CODEC.self_register_builtins()
