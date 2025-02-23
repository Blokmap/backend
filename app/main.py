from fastapi import FastAPI

app = FastAPI(root_path="/api")


@app.get("/")
async def root():
    return {"message": "Hello World"}
