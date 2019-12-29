from dataclasses import dataclass


@dataclass
class _DesiredState:
    custom_cli_path: str = None
    ip: str = None
    location: str = None
    port: int = None
    started: bool = False


desired_state = _DesiredState()
