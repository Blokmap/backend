from app.routers import translation
from fastapi import FastAPI

from .routers import auth, user


app = FastAPI(root_path="/api", docs_url="/docs", redoc_url=None)

app.include_router(auth.router)
app.include_router(user.router)
app.include_router(translation.router)