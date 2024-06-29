using UnityEngine;

public class Door : MonoBehaviour
{
    [SerializeField] Animator animator;

    void Reset()
    {
        animator = GetComponentInChildren<Animator>();
    }

    public void Open()
    {
        animator.SetTrigger("open");
    }
}