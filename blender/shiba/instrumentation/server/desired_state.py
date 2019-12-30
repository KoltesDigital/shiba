from dataclasses import dataclass


@dataclass
class _DesiredState:
    connected: bool = False
    custom_cli_path: str = None
    ip: str = None
    location: str = None
    port: int = None


desired_state = _DesiredState()
