"""Portable identities; no Rust layout, wall clock or ancestor path encoding."""

from dataclasses import dataclass
import re
import secrets

U64_MAX = (1 << 64) - 1
U128_MAX = (1 << 128) - 1
ALPHABET = "0123456789abcdefghijklmnopqrstuvwxyz"


class ContractError(ValueError):
    pass


def identity(value):
    if type(value) is not bytes or len(value) != 16 or not any(value):
        raise ContractError("identity must be a nonzero 16-byte value")
    return value


def new_identity():
    while True:
        value = secrets.token_bytes(16)
        if any(value):
            return value


def parse_identity(text):
    if type(text) is not str or not re.fullmatch(r"[0-9a-f]{32}", text):
        raise ContractError("identity text must be 32 lowercase hex digits")
    return identity(bytes.fromhex(text))


def u64(value, zero=False):
    if type(value) is not int or not (0 if zero else 1) <= value <= U64_MAX:
        raise ContractError("out-of-range u64")
    return value.to_bytes(8, "big")


def base36(value):
    if type(value) is not int or value < 0:
        raise ContractError("invalid base36 input")
    if value == 0:
        return "0"
    result = ""
    while value:
        value, digit = divmod(value, 36)
        result = ALPHABET[digit] + result
    return result


def number(text, maximum, zero=False):
    if (type(text) is not str or len(text) > 25
            or not re.fullmatch(r"0|[1-9a-z][0-9a-z]*", text)):
        raise ContractError("non-canonical base36")
    value = int(text, 36)
    if not (0 if zero else 1) <= value <= maximum:
        raise ContractError("base36 overflow or reserved zero")
    return value


def node_key(value):
    return "k" + base36(int.from_bytes(identity(value), "big"))


def parse_node_key(text):
    if type(text) is not str or not text.startswith("k") or len(text) > 26:
        raise ContractError("invalid durable node key")
    return number(text[1:], U128_MAX).to_bytes(16, "big")


def short_alias(value):
    u64(value)
    return "n" + base36(value)


def parse_alias(text):
    if type(text) is not str or not text.startswith("n") or len(text) > 14:
        raise ContractError("invalid local alias")
    return number(text[1:], U64_MAX)


def branch_address(branch, ordinal):
    u64(branch)
    u64(ordinal, zero=True)
    return "b" + base36(branch) + ":" + base36(ordinal)


def parse_branch(text):
    if type(text) is not str or not text.startswith("b") or len(text) > 28:
        raise ContractError("invalid branch address")
    parts = text[1:].split(":")
    if len(parts) != 2:
        raise ContractError("invalid branch fields")
    return number(parts[0], U64_MAX), number(parts[1], U64_MAX, zero=True)


@dataclass(frozen=True)
class NodeReference:
    project: bytes
    history: bytes
    node: bytes

    def __post_init__(self):
        for value in (self.project, self.history, self.node):
            identity(value)

    def text(self):
        return "hp1:" + ":".join(v.hex() for v in (self.project, self.history, self.node))

    @classmethod
    def parse(cls, text):
        if type(text) is not str or len(text) != 102 or not text.startswith("hp1:"):
            raise ContractError("invalid reference envelope")
        parts = text[4:].split(":")
        if len(parts) != 3:
            raise ContractError("invalid reference fields")
        return cls(*(parse_identity(v) for v in parts))
