from datetime import datetime, timedelta, timezone
from typing import Annotated

from app.services.user import create_user
from fastapi import APIRouter, Depends, Form, HTTPException, Response, status
from fastapi.security import OAuth2PasswordRequestForm

from app.constants import ACCESS_TOKEN_EXPIRE_MINUTES, ACCESS_TOKEN_NAME
from app.deps.db import DbSessionDep
from app.models.token import Token
from app.schemas.user import UserCreate
from app.services.auth import authenticate_user, create_access_token


router = APIRouter(prefix="/auth")


@router.post("/signup", status_code=status.HTTP_201_CREATED)
async def signup(
    user: Annotated[UserCreate, Form()], session: DbSessionDep, response: Response
):
    # Create a new user in the database.
    user = create_user(session, user)

    # Create an access token for the user.
    # TODO: do we want to do this? Or have the user log in manually after signing up? 
    access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = create_access_token(
        data={"sub": str(user.id)}, expires_delta=access_token_expires
    )

    # Set the access token as a cookie in the response.
    expires = datetime.now(timezone.utc) + timedelta(
        minutes=ACCESS_TOKEN_EXPIRE_MINUTES
    )

    response.set_cookie(
        key=ACCESS_TOKEN_NAME,
        value=access_token,
        expires=expires,
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
    # Authenticate the user with the provided credentials.
    user = authenticate_user(login_data.username, login_data.password, session)

    if user is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid username or password",
            headers={"WWW-Authenticate": "Bearer"},
        )

    # Create an access token for the user.
    access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = create_access_token(
        data={"sub": str(user.id)}, expires_delta=access_token_expires
    )

    # Return the access token.
    response = Response(status_code=status.HTTP_200_OK)

    response.set_cookie(
        key=ACCESS_TOKEN_NAME,
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
