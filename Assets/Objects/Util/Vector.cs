using UnityEngine;

public static class VectorExtensions
{
    public static Vector3 ProjectXZ(this Vector3 v) => new(v.x, 0, v.z);
}