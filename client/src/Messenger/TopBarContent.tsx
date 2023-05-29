import { Box, Avatar, Typography } from "@mui/material";

function TopBarContent() {
  return (
    <Box display="flex" alignItems="center">
      <Avatar
        sx={{
          width: 48,
          height: 48,
        }}
        alt="user.name"
      />
      <Box ml={1}>
        <Typography>Zain Baptista</Typography>
      </Box>
    </Box>
  );
}

export default TopBarContent;
