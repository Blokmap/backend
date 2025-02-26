from typing import Annotated

import jwt
from fastapi import Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer
from jwt.exceptions import InvalidTokenError
from sqlmodel import Session

from .constants import JWT_ALGORITHM, JWT_SECRET_KEY
from .database import get_session
from .models.user import User
from .models.token import TokenData


oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/auth/login")
TokenDep = Annotated[str, Depends(oauth2_scheme)]

DbSessionDep = Annotated[Session, Depends(get_session)]


async def get_user_session(token: TokenDep, session: DbSessionDep):
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
        payload = jwt.decode(token, JWT_SECRET_KEY, algorithms=[JWT_ALGORITHM])
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
