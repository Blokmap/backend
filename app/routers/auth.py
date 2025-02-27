from datetime import datetime, timedelta, timezone
from typing import Annotated

from fastapi import APIRouter, Depends, Form, HTTPException, Response, status
from fastapi.security import OAuth2PasswordRequestForm

from app.constants import ACCESS_TOKEN_EXPIRE_MINUTES
from app.dependencies.database import DbSessionDep
from app.models.user import User
from app.models.token import Token
from app.schemas.user import NewUser
from app.services.auth import authenticate_user, create_access_token, hash_user_password


router = APIRouter(prefix="/auth")


@router.post("/signup", status_code=status.HTTP_201_CREATED)
async def signup(
    signup_data: Annotated[NewUser, Form()], session: DbSessionDep, response: Response
) -> User:
    hashed_password = hash_user_password(signup_data.password)
    new_user = User(
        username=signup_data.username,
        email=signup_data.email,
        hashed_password=hashed_password,
    )

    user = new_user.save(session)

    if not user:
        raise HTTPException(status.HTTP_500_INTERNAL_SERVER_ERROR)

    access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = create_access_token(
        data={"sub": str(user.id)}, expires_delta=access_token_expires
    )

    response.set_cookie(
        key="access_token",
        value=access_token,
        expires=datetime.now(timezone.utc)
        + timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES),
        path="/",
        domain="",
        secure=True,
        httponly=True,
        samesite="lax",
    )

    return user


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

    response = Response(status_code=status.HTTP_200_OK)

    response.set_cookie(
        key="access_token",
        value=access_token,
        expires=datetime.now(timezone.utc)
        + timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES),
        path="/",
        domain="",
        secure=True,
        httponly=True,
        samesite="lax",
    )

    return response
