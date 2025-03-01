from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse

from app.exceptions import BlokmapApiError
from app.routers import auth, translation, user

app = FastAPI(root_path="/api", docs_url="/docs", redoc_url=None)


# Include the routers.
app.include_router(auth.router)
app.include_router(user.router)
app.include_router(translation.router)


# Include the exception handlers.
@app.exception_handler(BlokmapApiError)
async def api_error_handler(_: Request, exc: BlokmapApiError):
    return JSONResponse(
        status_code=exc.status_code,
        content={
            "detail": exc.message,
        },
    )
