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
  const [initializing, setInitializing] = useState(false);
  const [userKey, setUserKey] = useState<string | null>(null);
  const theme = useTheme();

  useEffect(() => {
    listen<string>("tor-change-initialized", (event) => {
      setUserKey(event.payload);
      setInitializing(false);
    });
  }, []);

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
        {initializing ? (
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
          <MiddleBlock setInitializing={setInitializing} userKey={userKey}/>
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
