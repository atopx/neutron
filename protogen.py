import subprocess


# 定义源文件路径和目标路径
src_path = "./library/proto/*.proto"
dst_path = "."

# 构建 protoc 命令
protoc_command = [
    "protoc",
    f"--go_out={dst_path}",
    "--go_opt=paths=source_relative",
    f"--go-grpc_out={dst_path}",
    "--go-grpc_opt=paths=source_relative",
    src_path,
]

try:
    # 执行 protoc 命令
    subprocess.run(protoc_command, check=True)
    print("Protobuf 生成成功！")
except subprocess.CalledProcessError as e:
    print(f"Protobuf 生成失败: {e}")
