from typing import Annotated

from fastapi import Depends, FastAPI, HTTPException, status
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm

from .model import User

from .routers import auth, user


app = FastAPI(root_path="/api", docs_url="/docs", redoc_url=None)

app.include_router(auth.router)
app.include_router(user.router)
