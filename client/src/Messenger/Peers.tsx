import {
  Stack,
  Box,
  Typography,
  Avatar,
  useTheme,
  Button,
  TableContainer,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
  Tooltip,
  Card,
} from "@mui/material";
import RefreshIcon from "@mui/icons-material/Refresh";
import { useEffect, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";

type Peer = {
  ip: string;
  port: number;
  id_key: string;
};

export default function Peers() {
  const theme = useTheme();
  const [peers, setPeers] = useState<Peer[]>([
    { id_key: "zrezkzezl", ip: "127.0.0.1", port: 69696969 },
    { id_key: "zrezkzezl", ip: "127.0.0.1", port: 69696969 },
    { id_key: "zrezkzezl", ip: "127.0.0.1", port: 69696969 },
    { id_key: "zrezkzezl", ip: "127.0.0.1", port: 69696969 },
    { id_key: "zrezkzezl", ip: "127.0.0.1", port: 69696969 },
    { id_key: "zrezkzezl", ip: "127.0.0.1", port: 69696969 },
    { id_key: "zrezkzezl", ip: "127.0.0.1", port: 69696969 },
    { id_key: "zrezkzezl", ip: "127.0.0.1", port: 69696969 },
  ]);

  useEffect(() => {
    listen<string>("tor-change-fetch-relays", (event) => {
      setPeers(JSON.parse(event.payload) as Peer[]);
    });
  }, []);

  const fetchRelays = () => {
    emit("tor-event", { event: "fetch-relays" });
  };

  return (
    <>
      <Box
        padding={2.5}
        sx={{ border: `${theme.colors.alpha.black[50]} solid 2px` }}
      >
        <Stack
          direction={"row"}
          gap={1}
          display={"flex"}
          alignItems={"center"}
          justifyContent={"space-between"}
        >
          <Stack direction={"row"} gap={1}>
            <Avatar />
            <Box display={"flex"} alignItems={"center"}>
              <Typography variant="h5" noWrap>
                {"user.name"}
              </Typography>
            </Box>
          </Stack>
          <Tooltip title="fetch relays" arrow>
            <Button variant="contained" onClick={fetchRelays}>
              <RefreshIcon />
            </Button>
          </Tooltip>
        </Stack>
      </Box>

      <Box m={2}>
        <Typography variant="h3" my={1} color={"white"}>
          Relays
        </Typography>
        <TableContainer>
          <Table sx={{ minWidth: 650, border: "white solid 1px" }}>
            <TableHead>
              <TableRow>
                <TableCell>IP</TableCell>
                <TableCell align="right">Port</TableCell>
                <TableCell align="right">Id Key</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {peers.map((peer) => (
                <TableRow
                  hover
                  key={peer.id_key}
                  sx={{ "&:last-child td, &:last-child th": { border: 0 } }}
                >
                  <TableCell component="th" scope="row">
                    {peer.ip}
                  </TableCell>
                  <TableCell align="right">{peer.port}</TableCell>
                  <TableCell align="right">{peer.id_key}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Box>
    </>
  );
}
