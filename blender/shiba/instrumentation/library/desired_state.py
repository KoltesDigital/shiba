from dataclasses import dataclass


@dataclass
class _DesiredState:
    loaded: bool = False
    path: str = None


desired_state = _DesiredState()
