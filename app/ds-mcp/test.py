import requests
import json

# 服务地址 - 如果更改了端口需要相应调整
BASE_URL = "http://localhost:3000"

def test_mcp_service():
    # 健康检查
    health = requests.get(f"{BASE_URL}/health")
    print(f"健康检查: {health.status_code} - {health.text}")
    
    # 获取服务清单
    print("\n获取服务清单:")
    response = requests.get(f"{BASE_URL}/manifest")
    manifest = response.json()
    print(json.dumps(manifest, indent=2, ensure_ascii=False))
    
    # 测试加法能力
    print("\n测试加法能力:")
    add_request = {
        "id": "1",
        "method": "add",
        "params": {"a": 5, "b": 3.5}
    }
    response = requests.post(f"{BASE_URL}/mcp", json=add_request)
    result = response.json()
    print(json.dumps(result, indent=2))
    
    # 测试系统信息能力
    print("\n测试系统信息能力:")
    sys_request = {
        "id": "2",
        "method": "getSystemInfo",
        "params": {}
    }
    response = requests.post(f"{BASE_URL}/mcp", json=sys_request)
    result = response.json()
    print(json.dumps(result, indent=2))
    
    # 测试文本处理能力
    print("\n测试文本处理能力:")
    text_request = {
        "id": "3",
        "method": "processText",
        "params": {"text": "Hello MCP Protocol!"}
    }
    response = requests.post(f"{BASE_URL}/mcp", json=text_request)
    result = response.json()
    print(json.dumps(result, indent=2))
    
    # 测试错误请求
    print("\n测试错误请求:")
    error_request = {
        "id": "4",
        "method": "unknownMethod",
        "params": {}
    }
    response = requests.post(f"{BASE_URL}/mcp", json=error_request)
    result = response.json()
    print(json.dumps(result, indent=2))

if __name__ == "__main__":
    test_mcp_service()