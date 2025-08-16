
import subprocess
import re
import sys

def get_cuda_version():
    try:
        # Run nvidia-smi and capture output
        result = subprocess.run(["nvidia-smi"], capture_output=True, text=True)
        output = result.stdout
        # Look for CUDA Version in the output
        match = re.search(r"CUDA Version: (\d+\.\d+)", output)
        if match:
            return match.group(1)
    except Exception as e:
        print("Could not detect CUDA version:", e)
    return None

cuda_version = get_cuda_version()
if cuda_version is None:
    print("No CUDA GPU found. Installing CPU-only PyTorch.")
    cmd = [sys.executable, "-m", "pip", "install", "torch", "torchvision", "torchaudio"]
else:
    # Convert version: 12.8 -> 128
    cu_short = cuda_version.replace(".", "")
    print(f"Detected CUDA version: {cuda_version} -> cu{cu_short}")
    url = f"https://download.pytorch.org/whl/cu{cu_short}"
    cmd = [sys.executable, "-m", "pip", "install", "torch", "torchvision", "torchaudio", "--index-url", url]

print("Running:", " ".join(cmd))
subprocess.run(cmd)
