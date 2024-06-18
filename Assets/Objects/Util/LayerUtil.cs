using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public static class LayerUtil
{
    public static bool Contains(this LayerMask layerMask, int layer)
    {
        return ((1 << layer) & layerMask.value) > 0;
    }
}
