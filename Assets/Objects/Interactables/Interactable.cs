using System.Collections;
using System.Collections.Generic;
using UnityEditor;
using UnityEngine;

public class Interactable : MonoBehaviour
{
    [SerializeField]
    public Transform Front;

    [SerializeField] List<Renderer> toonRenderers = new();
    [SerializeField, Layer] int highlightedLayer;

    List<int> oldLayers = new();

    bool highlighted = false;

    void Reset()
    {
        toonRenderers = new(GetComponentsInChildren<Renderer>());
        highlightedLayer = LayerMask.NameToLayer("Highlighted");
    }

    public virtual void Interact() {}

    public void Highlight()
    {
        if (highlighted) return;
        highlighted = true;

        oldLayers.Clear();
        foreach (var r in toonRenderers)
        {
            oldLayers.Add(r.gameObject.layer);
            r.gameObject.layer = highlightedLayer;
        }
    }
    public void Unhighlight() {
        if(!highlighted) return;
        highlighted = false;

        for(int i = 0; i < toonRenderers.Count; i++)
        {
            var r = toonRenderers[i];
            var layer = oldLayers[i];

            r.gameObject.layer = layer;
        }
    }
}
