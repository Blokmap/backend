from datetime import datetime, timedelta, timezone
from typing import Annotated

import jwt
from fastapi import APIRouter, Depends, Form, HTTPException, status
from fastapi.security import OAuth2PasswordRequestForm
from passlib.context import CryptContext

from app.constants import ACCESS_TOKEN_EXPIRE_MINUTES, JWT_ALGORITHM, JWT_SECRET_KEY
from app.dependencies import DbSessionDep
from app.models.user import User, NewUser
from app.models.token import Token


router = APIRouter(prefix="/auth")


pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


@router.post("/signup", status_code=status.HTTP_201_CREATED)
async def signup(
    signup_data: Annotated[NewUser, Form()], session: DbSessionDep
) -> User:
    hashed_password = pwd_context.hash(signup_data.password)
    new_user = User(
        username=signup_data.username,
        email=signup_data.email,
        hashed_password=hashed_password,
    )

    session.add(new_user)
    session.commit()
    session.refresh(new_user)

    return new_user


def authenticate_user(
    username: str, password: str, session: DbSessionDep
) -> User | None:
    user = User.get_by_username(session, username)

    if not user:
        return None

    if not pwd_context.verify(password, user.hashed_password):
        return None

    return user


def create_access_token(
    data: dict, expires_delta: timedelta = timedelta(minutes=15)
) -> str:
    to_encode = data.copy()

    expire = datetime.now(timezone.utc) + expires_delta
    to_encode.update({"exp": expire})

    encoded_jwt = jwt.encode(to_encode, JWT_SECRET_KEY, algorithm=JWT_ALGORITHM)

    return encoded_jwt


@router.post("/login")
async def login(
    login_data: Annotated[OAuth2PasswordRequestForm, Depends()], session: DbSessionDep
) -> Token:
    user = authenticate_user(login_data.username, login_data.password, session)

    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid username or password",
            headers={"WWW-Authenticate": "Bearer"},
        )

    access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = create_access_token(
        data={"sub": str(user.id)}, expires_delta=access_token_expires
    )

    return Token(access_token=access_token, token_type="bearer")
