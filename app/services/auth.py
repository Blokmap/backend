from datetime import datetime, timedelta, timezone

import jwt
from passlib.context import CryptContext
from sqlmodel import Session

from app.constants import JWT_ALGORITHM, JWT_SECRET_KEY
from app.models.user import User


pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


def hash_user_password(password: str) -> str:
    return pwd_context.hash(password)


def create_access_token(
    data: dict, expires_delta: timedelta = timedelta(minutes=15)
) -> str:
    to_encode = data.copy()

    expire = datetime.now(timezone.utc) + expires_delta
    to_encode.update({"exp": expire})

    encoded_jwt = jwt.encode(to_encode, JWT_SECRET_KEY, algorithm=JWT_ALGORITHM)

    return encoded_jwt


def authenticate_user(username: str, password: str, session: Session) -> User | None:
    user = User.get_by_username(session, username)

    if not user:
        return None

    if not pwd_context.verify(password, user.hashed_password):
        return None

    return user
