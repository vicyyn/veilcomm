import {
  useTheme,
  Grid,
  CircularProgress,
  Typography,
  Stack,
  Button,
} from "@mui/material";
import Logs from "./LeftBlock/Logs";
import Peers from "./RightBlock/Peers";
import MiddleBlock from "./MiddleBlock/MiddleBlock";
import { useEffect, useState } from "react";
import { listen, emit } from "@tauri-apps/api/event";

export default function Messenger() {
  const [initializing, setInitializing] = useState(true);
  const [clientType, setClientType] = useState(false);
  const [initialized, setInitialized] = useState(false);
  const [userKey, setUserKey] = useState<string | null>(null);
  const theme = useTheme();

  useEffect(() => {
    listen<string>("tor-change-initialized", (event) => {
      setUserKey(event.payload);
      setInitializing(false);
    });
  }, []);

  const init = () => {
    emit("tor-event", "initialize-" + clientType.toString());
    setInitialized(true);
  };

  return (
    <Grid container sx={{ background: theme.colors.alpha.black[50] }}>
      <Grid
        item
        xs={3.5}
        sx={{
          height: "100vh",
          border: `${theme.colors.alpha.black[50]} solid 2px`,
        }}
      >
        <Logs />
      </Grid>
      <Grid item xs={4}>
        {!initialized ? (
          <Stack
            gap={3}
            display={"flex"}
            justifyContent={"center"}
            alignItems={"center"}
            height={"100vh"}
          >
            <Stack
              direction={"row"}
              display={"flex"}
              justifyContent={"center"}
              alignItems={"center"}
              gap={2}
            >
              <label className="switch">
                <input
                  type="checkbox"
                  onChange={() => setClientType((prev) => !prev)}
                />
                <span className="slider"></span>
              </label>
              <Typography mt={2.5}>{clientType ? "user1" : "user2"}</Typography>
            </Stack>
            <Button variant="contained" onClick={init}>
              Initialize
            </Button>
          </Stack>
        ) : initializing ? (
          <Stack
            gap={1}
            display={"flex"}
            justifyContent={"center"}
            alignItems={"center"}
            height={"100vh"}
          >
            <CircularProgress />
            <Typography>Initializing...</Typography>
          </Stack>
        ) : (
          <MiddleBlock />
        )}
        {}
      </Grid>
      <Grid
        item
        xs={4.5}
        sx={{
          height: "100vh",
          border: `${theme.colors.alpha.black[50]} solid 2px`,
        }}
      >
        <Peers userKey={userKey} />
      </Grid>
    </Grid>
  );
}
