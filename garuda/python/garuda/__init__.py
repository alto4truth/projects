from enum import Enum
from typing import Union, Optional, Dict, List, Tuple


class DomainType(Enum):
    TOP = "Top"
    BOTTOM = "Bottom"
    INTEGER = "Integer"
    POINTER = "Pointer"
    BOOLEAN = "Boolean"
    FLOAT = "Float"
    INTERVAL = "Interval"
    CONSTANT = "Constant"
    STRUCT = "Struct"
    ARRAY = "Array"
    GROUP = "Group"


class DomainOperator(Enum):
    JOIN = "Join"
    MEET = "Meet"
    WIDEN = "Widen"
    NARROW = "Narrow"
    ASSIGN = "Assign"
    ADD = "Add"
    SUB = "Sub"


class DomainState:
    def __init__(
        self,
        is_top: bool = True,
        is_bottom: bool = False,
        domain_type: DomainType = DomainType.TOP,
    ):
        self.is_top = is_top
        self.is_bottom = is_bottom
        self.domain_type = domain_type


class TopDomain:
    def __init__(self):
        self.is_top = True
        self.is_bottom = False

    def __str__(self):
        return "⊤"

    def get_state(self):
        return DomainState(self.is_top, self.is_bottom, DomainType.TOP)


class BooleanDomain:
    def __init__(self, value: Optional[bool] = None):
        if value is not None:
            self.value = value
            self.is_top = False
            self.is_bottom = False
        else:
            self.value = None
            self.is_top = True
            self.is_bottom = False

    def __str__(self):
        if self.is_top:
            return "⊤"
        if self.is_bottom:
            return "⊥"
        return str(self.value) if self.value is not None else "bool"

    def get_state(self):
        return DomainState(self.is_top, self.is_bottom, DomainType.BOOLEAN)


class IntegerDomain:
    def __init__(self, lower: Optional[int] = None, upper: Optional[int] = None):
        if lower is not None and upper is not None:
            self.lower_bound = lower
            self.upper_bound = upper
            self.is_top = False
            self.is_bottom = False
        else:
            self.lower_bound = -(2**63)
            self.upper_bound = 2**63 - 1
            self.is_top = True
            self.is_bottom = False

    def __str__(self):
        if self.is_top:
            return "⊤"
        if self.is_bottom:
            return "⊥"
        if self.lower_bound == self.upper_bound:
            return str(self.lower_bound)
        return f"[{self.lower_bound}, {self.upper_bound}]"

    def get_state(self):
        return DomainState(self.is_top, self.is_bottom, DomainType.INTEGER)


class PointerDomain:
    def __init__(self, address: Optional[int] = None):
        if address is not None:
            self.address = address
            self.is_null = False
            self.is_top = False
            self.is_bottom = False
        else:
            self.address = None
            self.is_null = False
            self.is_top = True
            self.is_bottom = False

    @staticmethod
    def null():
        p = PointerDomain()
        p.is_null = True
        p.is_top = False
        return p

    def __str__(self):
        if self.is_top:
            return "⊤"
        if self.is_bottom:
            return "⊥"
        if self.is_null:
            return "null"
        if self.address is not None:
            return f"ptr({self.address})"
        return "ptr"

    def get_state(self):
        return DomainState(self.is_top, self.is_bottom, DomainType.POINTER)


class IntervalDomain:
    def __init__(self, lower: Optional[int] = None, upper: Optional[int] = None):
        if lower is not None and upper is not None:
            self.lower = lower
            self.upper = upper
            self.is_top = False
            self.is_bottom = False
        else:
            self.lower = -(2**63)
            self.upper = 2**63 - 1
            self.is_top = True
            self.is_bottom = False

    def __str__(self):
        if self.is_top:
            return "⊤"
        if self.is_bottom:
            return "⊥"
        if self.lower == self.upper:
            return str(self.lower)
        return f"[{self.lower}, {self.upper}]"

    def get_state(self):
        return DomainState(self.is_top, self.is_bottom, DomainType.INTERVAL)


class FloatDomain:
    def __init__(self, lower: Optional[float] = None, upper: Optional[float] = None):
        if lower is not None and upper is not None:
            self.lower_bound = lower
            self.upper_bound = upper
            self.is_top = False
            self.is_bottom = False
        else:
            self.lower_bound = float("-inf")
            self.upper_bound = float("inf")
            self.is_top = True
            self.is_bottom = False

    def __str__(self):
        if self.is_top:
            return "⊤"
        if self.is_bottom:
            return "⊥"
        if self.lower_bound == self.upper_bound:
            return str(self.lower_bound)
        return f"[{self.lower_bound}, {self.upper_bound}]"

    def get_state(self):
        return DomainState(self.is_top, self.is_bottom, DomainType.FLOAT)


ConstantValue = Union[int, float, bool, str]


class ConstantDomain:
    def __init__(self, value: Optional[ConstantValue] = None):
        if value is not None:
            self.value = value
            self.is_top = False
            self.is_bottom = False
        else:
            self.value = None
            self.is_top = True
            self.is_bottom = False

    def __str__(self):
        if self.is_top:
            return "⊤"
        if self.is_bottom:
            return "⊥"
        return str(self.value) if self.value is not None else "constant"

    def get_state(self):
        return DomainState(self.is_top, self.is_bottom, DomainType.CONSTANT)


Domain = Union[
    TopDomain,
    BooleanDomain,
    IntegerDomain,
    PointerDomain,
    IntervalDomain,
    FloatDomain,
    ConstantDomain,
]


class GroupDomain:
    def __init__(self, name: str = ""):
        self.name = name
        self.members: Dict[str, Domain] = {}
        self.is_top = not bool(name)
        self.is_bottom = False

    def add_member(self, key: str, domain: Domain):
        self.members[key] = domain
        self.is_top = False

    def get_member(self, key: str) -> Optional[Domain]:
        return self.members.get(key)

    def __str__(self):
        if self.is_top:
            return "⊤"
        if self.is_bottom:
            return "⊥"
        return f"Group({self.name})"

    def get_state(self):
        return DomainState(self.is_top, self.is_bottom, DomainType.GROUP)


class DomainFactory:
    @staticmethod
    def create(domain_type: DomainType) -> Domain:
        mapping = {
            DomainType.TOP: TopDomain,
            DomainType.BOOLEAN: BooleanDomain,
            DomainType.INTEGER: IntegerDomain,
            DomainType.POINTER: PointerDomain,
            DomainType.INTERVAL: IntervalDomain,
            DomainType.FLOAT: FloatDomain,
            DomainType.CONSTANT: ConstantDomain,
            DomainType.GROUP: GroupDomain,
        }
        return mapping.get(domain_type, TopDomain)()


AnalysisResult = Dict[str, any]


class GarudaAnalysis:
    def __init__(self, name: str = ""):
        self.name = name
        self.converged = False
        self.iteration_count = 0

    def analyze(self) -> AnalysisResult:
        return {
            "violations": [],
            "warnings": [],
            "converged": self.converged,
            "iteration_count": self.iteration_count,
        }


class ProgressUpdateTracker:
    def __init__(self):
        self.updates: List[Dict[str, any]] = []

    def add_update(self, stage: str, message: str, percentage: float):
        self.updates.append(
            {"stage": stage, "message": message, "percentage": percentage}
        )
