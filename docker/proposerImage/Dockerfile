FROM python:3.6
COPY . /propose
RUN pip install -r /propose/requirements.txt  -i https://pypi.tuna.tsinghua.edu.cn/simple
ENTRYPOINT ["python", "/propose/proposer.py"]
