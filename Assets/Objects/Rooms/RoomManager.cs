using System.Collections;
using System.Threading.Tasks;
using IPC;
using UnityEngine;
using UnityEngine.SceneManagement;

public class RoomManager : SingletonMonoBehaviour<RoomManager>
{
    [SerializeField] ComputersSO computers;
    [SerializeField] ComputerID startComputerID = ComputerID.First;
    [SerializeField] GameObject loadingScreen;

    Scene? loadedRoom = null;

    void Start(){
#if UNITY_EDITOR
        if(Room.Instance){
            loadedRoom = Room.Instance.gameObject.scene;
        }

        if(!loadedRoom.HasValue){
            LoadRoom(startComputerID);
        }
#else
        LoadRoom(startComputerID);
#endif
    }

    public void LoadRoom(ComputerID computerId){
        LoadRoomAsync(computerId);
    }

    public Task LoadRoomAsync(ComputerID computerID){
        var sceneName = ComputersSO.Instance.FindSceneName(computerID);
        return LoadSceneAndTeleportAsync(sceneName, computerID);
    }

    Task LoadSceneAndTeleportAsync(string sceneName, ComputerID computerID){
        var tcs = new TaskCompletionSource<bool>();

        void LoadImpl(){
            var loadOp = SceneManager.LoadSceneAsync(sceneName, LoadSceneMode.Additive);
            loadOp.completed += (op) => {
                try {
                    loadedRoom = SceneManager.GetSceneByName(sceneName);
                    loadingScreen.SetActive(false);
                    Player.Instance.Navigation.TeleportTo(computerID);
                    // cut camera
                    Player.Instance.MainVCam.CancelDamping();
                } finally {
                    tcs.SetResult(true);
                }
            };
        }

        loadingScreen.SetActive(true);
        if(loadedRoom is Scene loadedRoomNN){
            var unloadOp = SceneManager.UnloadSceneAsync(loadedRoomNN);
            unloadOp.completed += (op) => {
                loadedRoom = null;
                LoadImpl();
            };
        }
        else {
            LoadImpl();
        }

        return tcs.Task;
    }
}
