from pydantic import BaseModel, ConfigDict


def to_camel(string: str) -> str:
    """
    Convert a snake_case string to camelCase.
    Args:
        string (str): The snake_case string to convert.
    Returns:
        str: The converted camelCase string.
    """
    components = string.split("_")
    return components[0] + "".join(x.title() for x in components[1:])


class BaseModel(BaseModel):
    """Base Pydantic model with camel case conversion enabled."""

    config = ConfigDict(alias_generator=to_camel, populate_by_name=True)
