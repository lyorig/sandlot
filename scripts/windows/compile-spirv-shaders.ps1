<#
    .SYNOPSIS
    Compiles HLSL source files into SPIR-V.

    .DESCRIPTION
    Compiles HLSL shaders into SPIR-V for use with sandlot's GPU examples on Windows.
    Requires the Visual Studio Build Tools, and a `dxc[.exe]` that can be located by the script.
    To find where `dxc` is on your system, open the Native Tools (or Developer) Command Prompt, and run `which dxc`.

    .NOTES
    This currently has no use on Windows, as GPU examples use DXIL.
#>

$DxcExists = [bool] (Get-Command dxc -ErrorAction Ignore)

if ($DxcExists)
{
    Write-Verbose "dxc found on PATH"
} else
{
    $DxcFallbackDir = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64"
    $DxcExists = [bool] (Get-Command "$DxcFallbackDir\dxc.exe" -ErrorAction Ignore)

    if ($DxcExists)
    {
        Write-Verbose "dxc found in fallback dir ($DxcFallbackDir)"
        Set-Alias -Name dxc -Value "$DxcFallbackDir\dxc.exe"
    } else
    {
        Write-Host @"
dxc not found. Make sure that:
- you have the Visual Studio Build Tools installed
- dxc is located either:
    - on your PATH
    - in the directory specified by `$DxcFallbackDir

To find where `dxc` is on your system, you can open the Native Tools (or Developer) Command Prompt, and run `which dxc`.
Either add the containing directory to your PATH, or replace `$DxcFallbackDir with it.
"@
        return
    }
}

# `examples/gpu.rs`
dxc -spirv -O3 -T vs_6_0 -E vs_main examples/shaders/src/triangle.hlsl -Fo examples/shaders/triangle_vs.spv
dxc -spirv -O3 -T ps_6_0 -E fs_main examples/shaders/src/triangle.hlsl -Fo examples/shaders/triangle_fs.spv

# `examples/teapot.rs`
dxc -spirv -O3 -T vs_6_0 -E vs_main examples/shaders/src/teapot.hlsl -Fo examples/shaders/teapot_vs.spv
dxc -spirv -O3 -T ps_6_0 -E fs_main examples/shaders/src/teapot.hlsl -Fo examples/shaders/teapot_fs.spv
