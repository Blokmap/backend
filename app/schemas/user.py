from pydantic import BaseModel


class NewUser(BaseModel):
    username: str
    email: str
    password: str
