from fastapi import FastAPI

app = FastAPI(
    root_path="/api",
    docs_url=None,
    redoc_url=None
)

@app.get("/")
async def root():
    return {"message": "Hello World"}
