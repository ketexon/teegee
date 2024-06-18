using UnityEngine;
using System.Collections.Generic;

#if UNITY_EDITOR
using UnityEditor;

[CustomPropertyDrawer(typeof(LayerAttribute))]
public class LayerDrawer : PropertyDrawer
{
    (string[], int) GetLayerNames(int curLayer)
    {
        int index = 0;
        List<string> layers = new();
        for(int i = 0; i < 32; ++i)
        {
            var s = LayerMask.LayerToName(i);
            if (!string.IsNullOrEmpty(s))
            {
                if(i == curLayer)
                {
                    index = layers.Count;
                }
                layers.Add(s);
            }

        }
        return (layers.ToArray(), index);
    }

    public override void OnGUI(Rect position, SerializedProperty serializedProperty, GUIContent label)
    {
        var width = position.width;
        position.width = EditorGUIUtility.labelWidth;
        EditorGUI.PrefixLabel(position, label);

        position.x += EditorGUIUtility.labelWidth;
        position.width = width - EditorGUIUtility.labelWidth;

        var layerInt = serializedProperty.intValue;
        var (layerNames, layerIndex) = GetLayerNames(layerInt);

        EditorGUI.BeginChangeCheck();
        layerIndex = EditorGUI.Popup(position, layerIndex, layerNames);
        if (EditorGUI.EndChangeCheck())
            serializedProperty.intValue = LayerMask.NameToLayer(layerNames[layerIndex]);
    }
}
#endif

public class LayerAttribute : PropertyAttribute { }