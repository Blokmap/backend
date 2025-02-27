from typing import Annotated

import jwt
from fastapi import Cookie, Depends, HTTPException, status
from jwt.exceptions import InvalidTokenError
from sqlmodel import Session

from .constants import JWT_ALGORITHM, JWT_SECRET_KEY
from .database import get_session
from .models.user import User
from .models.token import TokenData


DbSessionDep = Annotated[Session, Depends(get_session)]


async def get_user_session(
    access_token: Annotated[str, Cookie()], session: DbSessionDep
):
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Invalid username or password",
        headers={"WWW-Authenticate": "Bearer"},
    )

    malformed_token_exception = HTTPException(
        status_code=status.HTTP_400_BAD_REQUEST,
        detail="Could not valid credentials",
    )

    try:
        payload = jwt.decode(access_token, JWT_SECRET_KEY, algorithms=[JWT_ALGORITHM])
        id = payload.get("sub")

        if id is None:
            raise credentials_exception

        token_data = TokenData(id=id)
    except InvalidTokenError:
        raise malformed_token_exception

    user = User.get(session, token_data.id)

    if user is None:
        raise credentials_exception

    return user


UserSessionDep = Annotated[User, Depends(get_user_session)]
