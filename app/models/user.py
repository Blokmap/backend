from typing import Self

from pydantic import BaseModel
from sqlmodel import Field, SQLModel, Session, select


class User(SQLModel, table=True):
    id: int = Field(primary_key=True)
    username: str
    email: str
    hashed_password: str = Field(exclude=True)

    @staticmethod
    def get(session: Session, id: int) -> Self | None:
        return session.get(User, id)

    @staticmethod
    def get_by_username(session: Session, username: str) -> Self | None:
        return session.exec(
            select(User).where(User.username == username).limit(1)
        ).first()


class NewUser(BaseModel):
    username: str
    email: str
    password: str


class InsertableUser(SQLModel, table=True):
    __tablename__ = "user"
    __table_args__ = {"extend_existing": True}

    username: str
    email: str
    hashed_password: str

    def save(self, session: Session) -> User:
        session.add(self)
        session.commit()

        return User.get_by_username(session, self.username)
