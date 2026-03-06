from dataclasses import dataclass, field
from typing import Optional, List, Any


@dataclass
class WrCall:
    tripod_name: str
    func_name: str
    params: str
    chain_id: int = 0
    topic: str = ""
    lei_price: int = 0
    tips: int = 0

    def to_dict(self) -> dict:
        d = {
            "tripod_name": self.tripod_name,
            "func_name": self.func_name,
            "params": self.params,
        }
        if self.chain_id:
            d["chain_id"] = self.chain_id
        if self.topic:
            d["topic"] = self.topic
        if self.lei_price:
            d["lei_price"] = self.lei_price
        if self.tips:
            d["tips"] = self.tips
        return d


@dataclass
class RdCall:
    tripod_name: str
    func_name: str
    params: str
    block_hash: str = ""

    def to_dict(self) -> dict:
        d = {
            "tripod_name": self.tripod_name,
            "func_name": self.func_name,
            "params": self.params,
        }
        if self.block_hash:
            d["block_hash"] = self.block_hash
        return d


@dataclass
class Event:
    tripod_name: str = ""
    func_name: str = ""
    value: Any = None


@dataclass
class Receipt:
    tx_hash: str = ""
    caller: Optional[str] = None
    block_stage: str = ""
    block_hash: str = ""
    height: int = 0
    tripod_name: str = ""
    writing_name: str = ""
    lei_cost: int = 0
    events: List[Event] = field(default_factory=list)
    error: str = ""
