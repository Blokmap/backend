from typing import Annotated

from fastapi import Depends, FastAPI, HTTPException, status
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm

from .model import User


fake_users_db = {
    "bob": {
        "id": 1,
        "username": "bob",
        "email": "bob@example.com",
        "password": "fakehashedappel1",
    },
    "alice": {
        "id": 2,
        "username": "alice",
        "email": "alice@example.com",
        "password": "fakehashedappel2",
    },
}


def fake_hash_password(password: str):
    return "fakehashed" + password

def get_user(username: str) -> User | None:
    if username in fake_users_db:
        return User(**fake_users_db[username])


app = FastAPI(root_path="/api", docs_url=None, redoc_url=None)


oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")


def fake_decode_token(token):
    user = get_user(token)
    return user


async def decode_current_user(token: Annotated[str, Depends(oauth2_scheme)]):
    user = fake_decode_token(token)
    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid authentication credentials",
            headers={"WWW-Authenticate": "Bearer"},
        )

    return user


@app.post("/token")
async def login(form_data: Annotated[OAuth2PasswordRequestForm, Depends()]):
    user_dict = fake_users_db.get(form_data.username)

    if not user_dict:
        raise HTTPException(status_code=400, detail="Incorrect username or password")

    user = User(**user_dict)
    hashed_password = fake_hash_password(form_data.password)

    if not hashed_password == user.hashed_password:
        raise HTTPException(status_code=400, detail="Incorrect username or password")

    return {"access_token": user.username, "token_type": "bearer"}



@app.get("/user/me")
async def get_current_user(current_user: Annotated[User, Depends(decode_current_user)]):
    return current_user
